use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    fold::{self, Fold},
    parse_macro_input, parse_quote, parse_str,
    visit::{self, Visit},
    Block, FnArg, ItemFn, PatType, ReturnType, Signature
};

struct ArgCollectorVisit(Vec<PatType>);
impl<'ast> Visit<'ast> for ArgCollectorVisit {
    fn visit_pat_type(&mut self, pat: &'ast PatType) {
        self.0.push(pat.to_owned());
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
            parse_str::<FnArg>("mut __error_events: ::bevy_fallible::bevy_ecs::ResMut<::bevy_fallible::bevy_app::Events<::bevy_fallible::SystemErrorEvent>>")
            .ok()
            .and_then(|arg| if let FnArg::Typed(pat) = arg { Some(pat) } else { None })
            .unwrap();

        let pats = [&[injection], args_visit.0.as_slice()].concat();
        node.inputs = pats.into_iter().map(|pat| FnArg::from(pat)).collect();
        node
    }

    fn fold_block(&mut self, node: Block) -> Block {
        let sig = self.sig.as_ref().unwrap();
        let ident = format!("{}", sig.ident);
        let ret = match &sig.output {
            ReturnType::Default => panic!("fallible system should return Result"),
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
            parse_quote! { let mut __impl = || -> #ret { #body }; },
            parse_quote! {
                match __impl() {
                    Ok(_) => (),
                    Err(err) => {
                        __error_events.send(::bevy_fallible::SystemErrorEvent { system_name: #ident, error: err.into() });
                    }
                };
            },
        ];

        Block {
            brace_token: node.brace_token,
            stmts,
        }
    }
}

/// A function attribute to use with fallible systems.
/// By default, will convert a system that returns a Result to
/// system with no return value, and will propagate errors as bevy events.
///
/// ```no_run
/// #[fallible_system]
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
pub fn fallible_system(_attrs: TokenStream, code: TokenStream) -> TokenStream {
    let input = parse_macro_input!(code as ItemFn);
    let modified = fold::fold_item_fn(&mut ModifiedFold::default(), input);
    let gen = quote! { #modified };
    gen.into()
}
