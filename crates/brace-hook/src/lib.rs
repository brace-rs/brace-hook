use std::borrow::Borrow;

use dyn_clone::DynClone;

use self::registry::REGISTRY;

pub use inventory;

pub use self::error::Error;
pub use self::registry::Registry;

pub mod error;
pub mod registry;

pub fn invoke<T, B, A>(hook: B, args: A) -> T::Output
where
    T: Hook<A>,
    B: Borrow<T>,
{
    Hook::invoke(hook.borrow(), args)
}

pub fn invoke_all<A, O>(name: &'static str, args: A) -> Result<Vec<O>, Error>
where
    A: Copy + 'static,
    O: 'static,
{
    REGISTRY.invoke_all::<A, O>(name, args)
}

pub trait Hook<A>: DynClone {
    type Output;

    fn invoke(&self, args: A) -> Self::Output;
}

impl<T, O> Hook<()> for T
where
    T: Fn() -> O + Clone,
{
    type Output = O;

    fn invoke(&self, _: ()) -> O {
        (self)()
    }
}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => {
        impl<Func, Out, $($name,)+> Hook<($($name,)+)> for Func
        where
            Func: Fn($($name,)+) -> Out + Clone,
        {
            type Output = Out;

            #[allow(non_snake_case)]
            fn invoke(&self, args: ($($name,)+)) -> Self::Output {
                let ($($name,)+) = args;

                (self)($($name,)+)
            }
        }

        peel! { $($name,)+ }
    };
}

tuple! { T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, }

#[cfg(test)]
mod tests {
    use super::{invoke, Error, Hook, Registry};

    fn my_hook(a: &str, b: &str) -> String {
        format!("my_hook ({}, {})", a, b)
    }

    #[test]
    fn test_invoke() {
        assert_eq!(
            invoke(my_hook, ("hello", "world")),
            String::from("my_hook (hello, world)")
        );
        assert_eq!(
            my_hook.invoke(("hello", "world")),
            String::from("my_hook (hello, world)")
        );
    }

    #[test]
    fn test_invoke_all() {
        let mut registry = Registry::new();

        registry.insert("not_my_hook", my_hook);
        registry.insert("my_hook", my_hook);
        registry.insert("my_hook", |a: &str, b: &str| {
            format!("my_hook_2 ({}, {})", a, b)
        });

        let res: Vec<String> = registry.invoke_all("my_hook", ("hello", "world")).unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], "my_hook (hello, world)");
        assert_eq!(res[1], "my_hook_2 (hello, world)");

        let res: Result<Vec<String>, Error> =
            registry.invoke_all("hook_not_exists", ("hello", "world"));

        assert!(res.is_err());

        let res: Result<Vec<usize>, Error> = registry.invoke_all("my_hook", ("hello", "world"));

        assert!(res.is_err());

        let res: Result<Vec<String>, Error> =
            registry.invoke_all("my_hook", ("hello", "world", "!"));

        assert!(res.is_err());
    }

    macro_rules! tuple {
        () => ();
        ( $name:ident, $($rest:ident,)* ) => {
            paste::item! {
                #[allow(clippy::too_many_arguments)]
                fn [<my_hook $name>]($($rest: &'static str,)*) {}

                #[test]
                fn [<test_hook_args $name>]() {
                    [<my_hook $name>].invoke(($(stringify!($rest),)*))
                }
            }

            tuple! { $($rest,)* }
        };
    }

    tuple! { _11, _10, _9, _8, _7, _6, _5, _4, _3, _2, _1, _0, }
}
