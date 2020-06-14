use std::borrow::Borrow;

pub fn invoke<'a, T, B, A>(hook: B, args: A) -> T::Output
where
    T: Hook<A>,
    B: Borrow<T>,
{
    Hook::invoke(hook.borrow(), args)
}

pub trait Hook<A> {
    type Output;

    fn invoke(&self, args: A) -> Self::Output;
}

impl<T, O> Hook<()> for T
where
    T: Fn() -> O,
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
            Func: Fn($($name,)+) -> Out,
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
    use super::{invoke, Hook};

    fn my_hook(a: &str, b: &str) -> String {
        format!("my_hook ({}, {})", a, b)
    }

    #[test]
    fn test_hook_invoke() {
        assert_eq!(
            invoke(my_hook, ("hello", "world")),
            String::from("my_hook (hello, world)")
        );
        assert_eq!(
            my_hook.invoke(("hello", "world")),
            String::from("my_hook (hello, world)")
        );
    }

    macro_rules! tuple {
        () => ();
        ( $name:ident, $($rest:ident,)* ) => {
            paste::item! {
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
