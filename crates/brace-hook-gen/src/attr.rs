use syn::parse::{Error, ParseStream, Result};
use syn::{Attribute, Path, Token};

// #[hook_attr(crate = brace_hook_crate)]
// See https://github.com/dtolnay/inventory/issues/10
pub fn crate_path(attrs: &mut Vec<Attribute>, krate: Path) -> Result<Path> {
    let mut path = None;
    let mut errors: Option<Error> = None;

    attrs.retain(|attr| {
        if !attr.path.is_ident("hook_attr") {
            return true;
        }

        match attr.parse_args_with(|input: ParseStream| {
            input.parse::<Token![crate]>()?;
            input.parse::<Token![=]>()?;
            input.call(Path::parse_mod_style)
        }) {
            Ok(res) => path = Some(res),
            Err(err) => match &mut errors {
                None => errors = Some(err),
                Some(errors) => errors.combine(err),
            },
        }

        false
    });

    match errors {
        None => Ok(path.unwrap_or(krate)),
        Some(errors) => Err(errors),
    }
}
