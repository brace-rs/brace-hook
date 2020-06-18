use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::token::Paren;
use syn::visit_mut::VisitMut;
use syn::{
    braced, parenthesized, Attribute, BareFnArg, Block, FnArg, Generics, Ident, Pat, ReturnType,
    Token, Type, TypeTuple, Visibility,
};

use crate::lifetime::Lifetimes;

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

    pub fn args_lifetimes(&self) -> Result<Punctuated<FnArg, Token![,]>> {
        let mut args = self.inputs.clone();
        let mut lifetimes = Lifetimes::new("'life");

        for arg in args.iter_mut() {
            if let FnArg::Typed(arg) = arg {
                lifetimes.visit_type_mut(&mut arg.ty);
            }
        }

        Ok(args)
    }

    pub fn lifetimes(&self) -> Result<Generics> {
        let mut args = self.arg_types()?;
        let mut lifetimes = Lifetimes::new("'life");

        for arg in args.iter_mut() {
            lifetimes.visit_type_mut(&mut arg.ty);
        }

        Ok(lifetimes.generics())
    }

    pub fn returns(&self) -> Type {
        match &self.output {
            ReturnType::Type(_, ty) => ty.as_ref().clone(),
            ReturnType::Default => Type::Tuple(TypeTuple {
                paren_token: Paren::default(),
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
