use proc_macro2::TokenStream;
use quote::{format_ident, quote};

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
    let default_name = format_ident!("{}__default", name);

    let default = match input.block {
        Some(block) => quote! {
            #[allow(unused_variables, non_snake_case)]
            fn #default_name(#args) -> #ret #block

            #krate::inventory::submit! {
                #![crate = #krate]
                #name::new(#default_name, 0, true)
            }
        },
        None => quote!(),
    };

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name(Box<dyn Fn(#arg_types) #output>, i32, bool);

        impl #name {
            pub fn new<T>(hook: T, weight: i32, default: bool) -> Self
            where
                T: Fn(#arg_types) #output + 'static,
            {
                Self(Box::new(hook), weight, default)
            }

            pub fn invoke(#args) -> Vec<#ret> {
                let mut out = Vec::new();
                let mut hooks: Vec<&'static #name> = #krate::inventory::iter::<#name>
                    .into_iter()
                    .filter(|hook| !hook.2)
                    .collect();

                if hooks.is_empty() {
                    hooks = #krate::inventory::iter::<#name>
                        .into_iter()
                        .filter(|hook| hook.2)
                        .collect();
                }

                hooks.sort_by_key(|hook| hook.1);

                for hook in hooks {
                    out.push(hook.0(#arg_names));
                }

                out
            }
        }

        impl #krate::inventory::Collect for #name {
            #[inline]
            fn registry() -> &'static #krate::inventory::Registry<Self> {
                static REGISTRY: #krate::inventory::Registry<#name> = #krate::inventory::Registry::new();
                &REGISTRY
            }
        }

        #default
    }
}
