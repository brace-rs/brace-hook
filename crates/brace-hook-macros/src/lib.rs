use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Error, ItemFn, LitStr, Token};

mod kw {
    syn::custom_keyword!(name);
}

struct Args {
    pub name: Option<LitStr>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Args { name: None });
        }

        input.parse::<kw::name>()?;
        input.parse::<Token![=]>()?;

        let name: LitStr = input.parse()?;

        Ok(Args { name: Some(name) })
    }
}

fn fn_name(input: ItemFn) -> Option<String> {
    Some(input.sig.ident.to_string())
}

#[proc_macro_attribute]
pub fn hook(args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    let method = input.sig.ident.clone();

    let args = parse_macro_input!(args as Args);
    let name = match args.name {
        Some(name) => quote!(#name),
        None => match fn_name(input.clone()) {
            Some(name) => quote!(#name),
            None => {
                let msg = "use #[brace_hook::hook(name = \"...\")] to specify a name";

                return TokenStream::from(
                    Error::new_spanned(&input.sig.ident, msg).to_compile_error(),
                );
            }
        },
    };

    TokenStream::from(quote! {
        #input

        brace_hook::register! { #name, #method }
    })
}
