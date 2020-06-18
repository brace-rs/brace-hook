use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::{
    braced, parenthesized, Attribute, BareFnArg, Block, FnArg, Ident, Pat, ReturnType, Token, Type,
    TypeTuple, Visibility,
};

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

pub struct HookFnSignature {
    pub attrs: Vec<Attribute>,
    pub vis: Visibility,
    pub fn_token: Token![fn],
    pub ident: Ident,
    pub paren_token: Paren,
    pub inputs: Punctuated<FnArg, Token![,]>,
    pub output: ReturnType,
    pub block: Option<Block>,
}

impl HookFnSignature {
    pub fn arg_names(&self) -> Result<Punctuated<Pat, Token![,]>> {
        let mut args = Punctuated::new();

        for arg in &self.inputs {
            match arg {
                FnArg::Receiver(rec) => {
                    return Err(Error::new(
                        rec.self_token.span,
                        "unexpected method receiver",
                    ))
                }
                FnArg::Typed(pat) => {
                    args.push(pat.pat.as_ref().clone());
                }
            }
        }

        Ok(args)
    }

    pub fn arg_types(&self) -> Result<Punctuated<BareFnArg, Token![,]>> {
        let mut args = Punctuated::new();

        for arg in &self.inputs {
            match arg {
                FnArg::Receiver(rec) => {
                    return Err(Error::new(
                        rec.self_token.span,
                        "unexpected method receiver",
                    ))
                }
                FnArg::Typed(pat) => args.push(BareFnArg {
                    attrs: pat.attrs.clone(),
                    name: None,
                    ty: pat.ty.as_ref().clone(),
                }),
            }
        }

        Ok(args)
    }

    pub fn returns(&self) -> Type {
        match &self.output {
            ReturnType::Type(_, ty) => ty.as_ref().clone(),
            ReturnType::Default => Type::Tuple(TypeTuple {
                paren_token: Paren(Span::call_site()),
                elems: Punctuated::new(),
            }),
        }
    }
}

impl Parse for HookFnSignature {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse::<Visibility>()?;

        let fn_token: Token![fn] = input.parse()?;
        let ident: Ident = input.parse()?;

        let content;
        let paren_token = parenthesized!(content in input);

        let mut inputs = Punctuated::new();

        while !content.is_empty() {
            let attrs = content.call(Attribute::parse_outer)?;

            let mut arg: FnArg = content.parse()?;

            match &mut arg {
                FnArg::Typed(arg) => arg.attrs = attrs,
                FnArg::Receiver(receiver) => {
                    return Err(Error::new(
                        receiver.self_token.span,
                        "unexpected method receiver",
                    ));
                }
            }

            inputs.push_value(arg);

            if content.is_empty() {
                break;
            }

            let comma: Token![,] = content.parse()?;

            inputs.push_punct(comma);
        }

        let output: ReturnType = input.parse()?;

        let content;
        let brace_token = braced!(content in input);

        let block = if content.is_empty() {
            None
        } else {
            let stmts = content.call(Block::parse_within)?;

            Some(Block { brace_token, stmts })
        };

        Ok(Self {
            attrs,
            vis,
            fn_token,
            ident,
            paren_token,
            inputs,
            output,
            block,
        })
    }
}
