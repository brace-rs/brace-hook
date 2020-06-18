use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::signature::HookFnSignature;

pub fn expand(mut input: HookFnSignature) -> TokenStream {
    let krate = match crate::attr::crate_path(&mut input.attrs) {
        Ok(path) => path,
        Err(err) => return err.to_compile_error(),
    };

    let arg_names_tuple = match input.arg_names_tuple() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let arg_types = match input.arg_types() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let lifetimes = match input.lifetimes() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let args_lifetimes = match input.args_lifetimes() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let iter_arg_types = match input.iter_arg_types() {
        Ok(res) => res,
        Err(err) => return err.to_compile_error(),
    };

    let iter_arg_names = input.iter_arg_names();

    let ret = input.returns();
    let vis = input.vis;
    let args = input.inputs;
    let output = input.output;
    let name = input.ident;
    let default_name = format_ident!("{}__default", name);
    let iter_name = format_ident!("{}__iter", name);

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

            pub fn with #lifetimes (#args_lifetimes) -> #iter_name #lifetimes {
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

                #iter_name {
                    args: #arg_names_tuple,
                    hooks: hooks.into_iter(),
                }
            }
        }

        #[allow(non_camel_case_types)]
        #vis struct #iter_name #lifetimes {
            args: #iter_arg_types,
            hooks: std::vec::IntoIter<&'static #name>,
        }

        impl #lifetimes std::iter::Iterator for #iter_name #lifetimes {
            type Item = #ret;

            fn next(&mut self) -> Option<Self::Item> {
                match self.hooks.next() {
                    Some(hook) => Some(hook.0(#iter_arg_names)),
                    None => None,
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                self.hooks.size_hint()
            }
        }

        impl #lifetimes std::iter::DoubleEndedIterator for #iter_name #lifetimes {
            fn next_back(&mut self) -> Option<Self::Item> {
                match self.hooks.next_back() {
                    Some(hook) => Some(hook.0(#iter_arg_names)),
                    None => None,
                }
            }
        }

        impl #lifetimes std::iter::ExactSizeIterator for #iter_name #lifetimes {}

        impl #lifetimes std::iter::FusedIterator for #iter_name #lifetimes {}

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
