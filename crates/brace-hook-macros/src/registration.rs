use proc_macro2::TokenStream;
use syn::{ItemFn, LitInt, Path};

pub fn expand(path: Path, weight: LitInt, mut input: ItemFn) -> TokenStream {
    let name = input.sig.ident.clone();

    let krate = match crate::attr::crate_path(&mut input.attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    quote::quote! {
        #input

        #krate::inventory::submit! {
            #![crate = #krate]
            #path::new(#name, #weight, false)
        }
    }
}
