use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    fold::{self, Fold},
    parse_macro_input, parse_quote, parse_str,
    visit::{self, Visit},
    Block, FnArg, ItemFn, PatType, ReturnType, Signature, Type,
};

struct ArgCollectorVisit(Vec<PatType>);
impl<'ast> Visit<'ast> for ArgCollectorVisit {
    fn visit_pat_type(&mut self, pat: &'ast PatType) {
        self.0.push(pat.to_owned());
    }
}

fn is_command(pat: &PatType) -> bool {
    if let Type::Path(path) = pat.ty.as_ref() {
        path.path.is_ident("bevy_ecs::Commands") || path.path.is_ident("Commands")
    } else {
        false
    }
}

#[derive(Default)]
struct ModifiedFold {
    sig: Option<Signature>,
}
impl Fold for ModifiedFold {
    fn fold_signature(&mut self, mut node: Signature) -> Signature {
        self.sig = Some(node.clone());
        node.output = ReturnType::Default;

        let mut args_visit = ArgCollectorVisit(vec![]);
        visit::visit_signature(&mut args_visit, &node);
        let injection =
            parse_str::<FnArg>("mut __error_events: bevy_ecs::ResMut<bevy_app::Events<bevy_fallable::SystemErrorEvent>>")
            .ok()
            .and_then(|arg| if let FnArg::Typed(pat) = arg { Some(pat) } else { None })
            .unwrap();

        let pats = if let Some((commands, rest)) =
            args_visit.0.split_first().filter(|(p, _)| is_command(p))
        {
            [&[commands.to_owned(), injection], rest].concat()
        } else {
            [&[injection], args_visit.0.as_slice()].concat()
        };
        node.inputs = pats.into_iter().map(|pat| FnArg::from(pat)).collect();
        node
    }

    fn fold_block(&mut self, node: Block) -> Block {
        let sig = self.sig.as_ref().unwrap();
        let ident = format!("{}", sig.ident);
        let ret = match &sig.output {
            ReturnType::Default => panic!("fallable system should return Result"),
            ReturnType::Type(_, t) => t,
        };

        let body: proc_macro2::TokenStream =
            node.stmts
                .into_iter()
                .fold(proc_macro2::TokenStream::new(), |mut tokens, stmt| {
                    stmt.to_tokens(&mut tokens);
                    tokens
                });

        let stmts = vec![
            parse_quote! { let __res: #ret = { #body }; },
            parse_quote! {
                match __res {
                    Ok(_) => (),
                    Err(err) => {
                        __error_events.send(bevy_fallable::SystemErrorEvent { system_name: #ident, error: err.into() });
                    }
                };
            }
        ];


        Block {
            brace_token: node.brace_token,
            stmts,
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
#[proc_macro_attribute]
pub fn fallable_system(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    let input = parse_macro_input!(code as ItemFn);
    let modified = fold::fold_item_fn(&mut ModifiedFold::default(), input);
    let gen = quote! { #modified };
    gen.into()
}
