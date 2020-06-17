use proc_macro::TokenStream;
use syn::parse_macro_input;

use self::args::Args;

mod args;
mod attr;
mod declaration;
mod registration;

#[proc_macro_attribute]
pub fn hook(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as Args);

    TokenStream::from(match args {
        Args::None => declaration::expand(parse_macro_input!(input)),
        Args::With(path, weight) => registration::expand(path, weight, parse_macro_input!(input)),
    })
}
