use proc_macro2::Span;
use syn::parse::{Parse, ParseStream, Result};
use syn::{LitInt, Path, Token};

pub enum Args {
    None,
    With(Path, LitInt),
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Args::None)
        } else {
            let path = input.parse::<Path>()?;

            if input.is_empty() {
                return Ok(Self::With(path, LitInt::new("0", Span::call_site())));
            }

            input.parse::<Token![,]>()?;

            let weight = input.parse::<LitInt>()?;

            Ok(Self::With(path, weight))
        }
    }
}
