use std::iter::FromIterator;

use proc_macro2::Span;
use syn::punctuated::Punctuated;
use syn::token::{Gt, Lt};
use syn::visit_mut::{self, VisitMut};
use syn::{
    GenericArgument, GenericParam, Generics, Lifetime, LifetimeDef, Receiver, TypeReference,
};

pub struct Lifetimes {
    pub elided: Vec<Lifetime>,
    pub explicit: Vec<Lifetime>,
    pub name: &'static str,
}

impl Lifetimes {
    pub fn new(name: &'static str) -> Self {
        Lifetimes {
            elided: Vec::new(),
            explicit: Vec::new(),
            name,
        }
    }

    pub fn generics(&self) -> Generics {
        Generics {
            lt_token: Some(Lt::default()),
            params: Punctuated::from_iter(
                self.explicit
                    .iter()
                    .chain(&self.elided)
                    .filter(|lifetime| lifetime.ident != "static")
                    .map(|lifetime| {
                        GenericParam::Lifetime(LifetimeDef {
                            attrs: Vec::new(),
                            lifetime: lifetime.clone(),
                            colon_token: None,
                            bounds: Punctuated::new(),
                        })
                    }),
            ),
            gt_token: Some(Gt::default()),
            where_clause: None,
        }
    }

    fn visit_opt_lifetime(&mut self, lifetime: &mut Option<Lifetime>) {
        match lifetime {
            None => *lifetime = Some(self.next_lifetime()),
            Some(lifetime) => self.visit_lifetime(lifetime),
        }
    }

    fn visit_lifetime(&mut self, lifetime: &mut Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = self.next_lifetime();
        } else {
            self.explicit.push(lifetime.clone());
        }
    }

    fn next_lifetime(&mut self) -> Lifetime {
        let name = format!("{}{}", self.name, self.elided.len());
        let life = Lifetime::new(&name, Span::call_site());
        self.elided.push(life.clone());
        life
    }
}

impl VisitMut for Lifetimes {
    fn visit_receiver_mut(&mut self, arg: &mut Receiver) {
        if let Some((_, lifetime)) = &mut arg.reference {
            self.visit_opt_lifetime(lifetime);
        }
    }

    fn visit_type_reference_mut(&mut self, ty: &mut TypeReference) {
        self.visit_opt_lifetime(&mut ty.lifetime);
        visit_mut::visit_type_reference_mut(self, ty);
    }

    fn visit_generic_argument_mut(&mut self, gen: &mut GenericArgument) {
        if let GenericArgument::Lifetime(lifetime) = gen {
            self.visit_lifetime(lifetime);
        }

        visit_mut::visit_generic_argument_mut(self, gen);
    }
}
