use proc_macro::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Error, ItemFn, LitInt, LitStr, Token};

mod kw {
    syn::custom_keyword!(name);
    syn::custom_keyword!(weight);
}

struct Args {
    pub name: Option<LitStr>,
    pub weight: Option<LitInt>,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Args {
                name: None,
                weight: None,
            });
        }

        let mut name: Option<LitStr> = None;
        let mut weight: Option<LitInt> = None;

        if input.parse::<kw::name>().is_ok() {
            input.parse::<Token![=]>()?;

            name = Some(input.parse::<LitStr>()?);

            if input.is_empty() {
                return Ok(Args { name, weight });
            }
        }

        if name.is_some() {
            input.parse::<Token![,]>()?;
        }

        if input.parse::<kw::weight>().is_ok() {
            input.parse::<Token![=]>()?;

            weight = Some(input.parse::<LitInt>()?);

            if input.is_empty() {
                return Ok(Args { name, weight });
            }
        }

        Ok(Args { name, weight })
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

    match args.weight {
        Some(weight) => TokenStream::from(quote! {
            #input

            brace_hook::register! { #name, #method, #weight }
        }),
        None => TokenStream::from(quote! {
            #input

            brace_hook::register! { #name, #method }
        }),
    }
}
