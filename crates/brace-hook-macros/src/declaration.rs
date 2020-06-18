use proc_macro2::TokenStream;
use quote::quote;

use crate::signature::HookFnSignature;

pub fn expand(mut input: HookFnSignature) -> TokenStream {
    let krate = match crate::attr::crate_path(&mut input.attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let arg_names = match input.arg_names() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let arg_types = match input.arg_types() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let ret = input.returns();
    let vis = input.vis;
    let args = input.inputs;
    let output = input.output;
    let name = input.ident;

    let body = match input.block {
        Some(_) => quote! {
            if hooks.len() == 0 {
                out.push(Self::default(#arg_names));

                return out;
            }
        },
        None => quote!(),
    };

    let default = match input.block {
        Some(block) => quote! {
            #[allow(unused_variables)]
            fn default(#args) -> #ret #block
        },
        None => quote!(),
    };

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name(Box<dyn Fn(#arg_types) #output>, i32);

        impl #name {
            pub fn new<T>(hook: T, weight: i32) -> Self
            where
                T: Fn(#arg_types) #output + 'static,
            {
                Self(Box::new(hook), weight)
            }

            pub fn invoke(#args) -> Vec<#ret> {
                let mut out = Vec::new();
                let mut hooks: Vec<&'static #name> = #krate::inventory::iter::<#name>
                    .into_iter()
                    .collect();

                #body

                hooks.sort_by_key(|hook| hook.1);

                for hook in hooks {
                    out.push(hook.0(#arg_names));
                }

                out
            }

            #default
        }

        #krate::inventory::collect!(#name);
    }
}
