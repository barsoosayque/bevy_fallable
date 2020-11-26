use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    fold::{self, Fold},
    parse::{Parse, ParseStream},
    parse_macro_input, parse_quote,
    punctuated::Punctuated,
    token::Comma,
    visit::{self, Visit},
    Block, Error, Ident, ItemFn, Pat, Result, ReturnType, Signature,
};

#[derive(Clone, Copy)]
enum Mode {
    Replace,
    Keep,
}
impl Parse for Mode {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Ident>().map_or(Ok(Mode::Replace), |ident| {
            if ident == "keep" {
                Ok(Mode::Keep)
            } else {
                Err(Error::new(ident.span(), "unexpected attribute"))
            }
        })
    }
}

struct ArgCollectorVisit(Vec<Pat>);
impl<'ast> Visit<'ast> for ArgCollectorVisit {
    fn visit_pat(&mut self, pat: &'ast Pat) {
        self.0.push(pat.to_owned());
    }
}

struct OriginalFold {
    mode: Mode,
}
impl OriginalFold {
    fn new(mode: Mode) -> Self { Self { mode } }
}
impl Fold for OriginalFold {
    fn fold_signature(&mut self, mut node: Signature) -> Signature {
        match self.mode {
            Mode::Replace => node,
            Mode::Keep => {
                node.ident = Ident::new(&format!("{}_fallable", node.ident), node.ident.span());
                node
            }
        }
    }
}

struct ModifiedFold {
    mode: Mode,
    sig: Option<Signature>,
}
impl ModifiedFold {
    fn new(mode: Mode) -> Self { Self { mode, sig: None } }
}
impl Fold for ModifiedFold {
    fn fold_signature(&mut self, mut node: Signature) -> Signature {
        self.sig = Some(node.clone());
        node.output = ReturnType::Default;
        node
    }

    fn fold_block(&mut self, node: Block) -> Block {
        let sig = self.sig.as_ref().unwrap();
        let ret = match &sig.output {
            ReturnType::Default => panic!("fallable system should return something"),
            ReturnType::Type(_, t) => t
        };

        let mut block = match self.mode {
            Mode::Replace => {
                let body: proc_macro2::TokenStream = node.stmts.into_iter().fold(
                    proc_macro2::TokenStream::new(),
                    |mut tokens, stmt| {
                        stmt.to_tokens(&mut tokens);
                        tokens
                    },
                );

                vec![parse_quote! {
                    let r: #ret = { #body };
                }]
            }
            Mode::Keep => {
                let name = Ident::new(&format!("{}_fallable", sig.ident), sig.ident.span());
                let mut args_visit = ArgCollectorVisit(vec![]);
                visit::visit_signature(&mut args_visit, &sig);
                let args: Punctuated<Pat, Comma> = args_visit.0.iter().cloned().collect();

                vec![parse_quote! {
                    let r: #ret = #name(#args);
                }]
            }
        };
        block.push(parse_quote! {
            match r {
                Err(err) => {},
                Ok(_) => {}
            };
        });

        Block {
            brace_token: node.brace_token,
            stmts: block,
        }
    }
}

/// A function attribute to use with fallable systems.
/// By default, will convert a system that returns a Result to
/// system with no return value, and will propagate errors as bevy events.
///
/// ```no_run
/// #[fallable_system]
/// fn system(asset_server: Res<AssetServer>) -> anyhow::Result<()> {
///     let handle: Handle<Texture> = asset_server.load("texture")?;
/// }
///
/// fn main() {
///     App::build()
///         .add_startup_system(system.system())
///         .run();
/// }
/// ```
///
/// If used with `#[fallable_system(keep)]`, will additionally add suffix
/// *_fallable* to the original function, so you will have two functions
///  as an output.
#[proc_macro_attribute]
pub fn fallable_system(attrs: TokenStream, code: TokenStream) -> TokenStream {
    let input = parse_macro_input!(code as ItemFn);
    let mode = parse_macro_input!(attrs);

    let original = fold::fold_item_fn(&mut OriginalFold::new(mode), input.clone());
    let modified = fold::fold_item_fn(&mut ModifiedFold::new(mode), input);

    let gen = match mode {
        Mode::Replace => quote! { #modified },
        Mode::Keep => quote! { #original #modified }
    };
    gen.into()
}
