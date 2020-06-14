use std::any::{Any, TypeId};
use std::collections::BTreeMap;

use super::error::Error;
use super::Hook;

pub struct Registry(BTreeMap<(&'static str, TypeId, TypeId), Entry>);

impl Registry {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert<T, A, O>(&mut self, name: &'static str, hook: T)
    where
        T: Hook<A, Output = O> + 'static,
        A: 'static,
        O: 'static,
    {
        let entry = self
            .0
            .entry((name, TypeId::of::<A>(), TypeId::of::<O>()))
            .or_insert_with(Entry::default);

        entry.insert(hook);
    }

    pub fn invoke_all<A, O>(&self, name: &'static str, args: A) -> Result<Vec<O>, Error>
    where
        A: Copy + 'static,
        O: 'static,
    {
        match self.0.get(&(name, TypeId::of::<A>(), TypeId::of::<O>())) {
            Some(entry) => Ok(entry.invoke_all(args)),
            None => Err(Error::message(format!(
                "No matching hooks found for {}",
                name
            ))),
        }
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Entry(Vec<Box<dyn Any>>);

impl Entry {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert<T, A, O>(&mut self, hook: T)
    where
        T: Hook<A, Output = O> + 'static,
        A: 'static,
        O: 'static,
    {
        self.0.push(Box::new(Anon::new(hook)))
    }

    pub fn invoke_all<A, O>(&self, args: A) -> Vec<O>
    where
        A: Copy + 'static,
        O: 'static,
    {
        let mut out = Vec::new();

        for item in &self.0 {
            if let Some(hook) = item.downcast_ref::<Anon<A, O>>() {
                out.push(hook.invoke(args));
            }
        }

        out
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Anon<A, O>(Box<dyn Hook<A, Output = O>>);

impl<A, O> Anon<A, O> {
    pub fn new<T>(hook: T) -> Self
    where
        T: Hook<A, Output = O> + 'static,
    {
        Self(Box::new(hook))
    }
}

impl<A, O> Hook<A> for Anon<A, O> {
    type Output = O;

    fn invoke(&self, args: A) -> O {
        self.0.invoke(args)
    }
}
