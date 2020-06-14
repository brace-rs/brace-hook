use std::any::{Any, TypeId};
use std::collections::BTreeMap;

use dyn_clone::{clone_box, clone_trait_object, DynClone};
use inventory::collect;
use once_cell::sync::Lazy;

use super::error::Error;
use super::Hook;

pub(crate) static REGISTRY: Lazy<Registry> = Lazy::new(Registry::default);

pub struct Registry(BTreeMap<(&'static str, TypeId, TypeId), Entry>);

impl Registry {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn insert<T, A, O>(&mut self, name: &'static str, hook: T)
    where
        T: Hook<A, Output = O> + Send + Sync + 'static,
        A: 'static,
        O: 'static,
    {
        let entry = self
            .0
            .entry((name, TypeId::of::<A>(), TypeId::of::<O>()))
            .or_insert_with(Entry::default);

        entry.insert(hook);
    }

    pub fn insert_record(&mut self, record: Record) {
        let entry = self.0.entry(record.0).or_insert_with(Entry::default);

        entry.insert_item(record.1);
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
        let mut this = Self::new();

        for record in inventory::iter::<Record> {
            this.insert_record(record.clone());
        }

        this
    }
}

pub struct Entry(Vec<Box<dyn MyAny>>);

impl Entry {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn insert<T, A, O>(&mut self, hook: T)
    where
        T: Hook<A, Output = O> + Send + Sync + 'static,
        A: 'static,
        O: 'static,
    {
        self.0.push(Box::new(Anon::new(hook)))
    }

    pub fn insert_item(&mut self, item: Box<dyn MyAny>) {
        self.0.push(item)
    }

    pub fn invoke_all<A, O>(&self, args: A) -> Vec<O>
    where
        A: Copy + 'static,
        O: 'static,
    {
        let mut out = Vec::new();

        for item in &self.0 {
            if let Some(hook) = item.as_any().downcast_ref::<Anon<A, O>>() {
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

pub trait MyAny: Any + DynClone + Send + Sync {
    fn as_any(&self) -> &dyn Any;
}

clone_trait_object!(MyAny);

pub struct Anon<A, O>(Box<dyn Hook<A, Output = O> + Send + Sync>);

impl<A, O> Anon<A, O> {
    pub fn new<T>(hook: T) -> Self
    where
        T: Hook<A, Output = O> + Send + Sync + 'static,
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

impl<A, O> Clone for Anon<A, O> {
    fn clone(&self) -> Self {
        Anon(clone_box(&*self.0))
    }
}

impl<A, O> MyAny for Anon<A, O>
where
    Self: 'static,
{
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
pub struct Record((&'static str, TypeId, TypeId), Box<dyn MyAny>);

impl Record {
    pub fn new<T, A>(name: &'static str, hook: T) -> Self
    where
        T: Hook<A> + Send + Sync + 'static,
        A: 'static,
    {
        Self(
            (name, TypeId::of::<A>(), TypeId::of::<T::Output>()),
            Box::new(Anon::new(hook)),
        )
    }
}

collect!(Record);

#[macro_export]
macro_rules! register {
    ( $name:tt, $hook:path ) => {
        $crate::inventory::submit! {
            #![crate = brace_hook]
            $crate::registry::Record::new($name, $hook)
        }
    };
}
