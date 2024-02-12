use proc_macro::TokenStream;
use quote::quote;
use syn::{Ident, ItemFn, LitStr, parse_macro_input, Path, Token};
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
struct HandlerArg {
    method: Path,
    url: LitStr,
}

impl Parse for HandlerArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let method;
        let method_ident: Path = input.parse()?;
        let method_str = method_ident.require_ident()?.to_string();
        match method_str.as_str() {
            "get" => {},
            "post" => {},
            "put" => {},
            "patch" => {},
            "delete" => {},
            other => {
                return Err(input.error( format!("unrecognized method: {}", other)));
            }
        }
        method = method_ident;
        input.parse::<Token![,]>()?;
        let url: LitStr = input.parse()?;
        Ok(HandlerArg{
            method,
            url
        })
    }
}

pub(crate) fn parse_handler(args: TokenStream, input: TokenStream) -> TokenStream{
    let mut input = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as HandlerArg);
    let method = args.method;
    let url = args.url;
    let old_fn_name = input.sig.ident;
    input.sig.ident = Ident::new("register", old_fn_name.span());
    (quote!{
        pub fn #old_fn_name() -> ::salvo::Router{

            #[::salvo::handler]
            // #[::tracing::instrument(fields(uri = req.uri().path(), method = req.method().as_str(), request_id=depot.get::<String>("request_id").unwrap_or(&String::from("unknown"))),skip_all)]
            #input
            ::salvo::Router::with_path(#url).#method(register)
        }
    }).into()
}