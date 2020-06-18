use std::iter::FromIterator;

pub fn invoke<I>(iter: I) -> Vec<I::Item>
where
    I: Iterator,
{
    iter.collect()
}

pub fn try_invoke<I, T, E>(iter: I) -> Result<Vec<T>, E>
where
    I: Iterator<Item = Result<T, E>>,
{
    Result::from_iter(iter)
}

pub fn exec<T>(iter: T)
where
    T: Iterator,
{
    iter.for_each(drop);
}

pub fn try_exec<I, T, E>(iter: I) -> Result<(), E>
where
    I: Iterator<Item = Result<T, E>>,
{
    for res in iter {
        if let Err(err) = res {
            return Err(err);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate self as brace_hook;

    use brace_hook::hook;

    #[hook]
    fn my_hook(input: &str) -> Result<String, String> {}

    #[hook(my_hook, 1)]
    fn my_hook_1(input: &str) -> Result<String, String> {
        Ok(format!("my_hook_1: {}", input))
    }

    #[hook(my_hook, 2)]
    fn my_hook_2(input: &str) -> Result<String, String> {
        Ok(format!("my_hook_2: {}", input))
    }

    #[test]
    fn test_invoke() {
        let res = hook::invoke(my_hook::with("hello"));

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], Ok(String::from("my_hook_1: hello")));
        assert_eq!(res[1], Ok(String::from("my_hook_2: hello")));
    }

    #[test]
    fn test_try_invoke_ok() {
        let res = hook::try_invoke(my_hook::with("hello"));

        assert!(res.is_ok());

        let res = res.unwrap();

        assert_eq!(res.len(), 2);
        assert_eq!(res[0], String::from("my_hook_1: hello"));
        assert_eq!(res[1], String::from("my_hook_2: hello"));
    }

    #[hook]
    fn my_bad_hook(input: &str) -> Result<String, String> {}

    #[hook(my_bad_hook, 1)]
    fn my_bad_hook_1(input: &str) -> Result<String, String> {
        Err(format!("my_bad_hook_1: {}", input))
    }

    #[hook(my_bad_hook, 2)]
    fn my_bad_hook_2(input: &str) -> Result<String, String> {
        Ok(format!("my_bad_hook_2: {}", input))
    }

    #[test]
    fn test_try_invoke_err() {
        let res = hook::try_invoke(my_bad_hook::with("hello"));

        assert!(res.is_err());
        assert_eq!(res, Err(String::from("my_bad_hook_1: hello")));
    }

    #[hook]
    fn my_mut_hook(output: &mut Vec<&str>) -> Result<(), String> {}

    #[hook(my_mut_hook, 1)]
    fn my_mut_hook_1(output: &mut Vec<&str>) -> Result<(), String> {
        output.push("my_mut_hook_1");
        Ok(())
    }

    #[hook(my_mut_hook, 2)]
    fn my_mut_hook_2(output: &mut Vec<&str>) -> Result<(), String> {
        output.push("my_mut_hook_2");
        Ok(())
    }

    #[test]
    fn test_exec() {
        let mut items = Vec::new();

        hook::exec(my_mut_hook::with(&mut items));

        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "my_mut_hook_1");
        assert_eq!(items[1], "my_mut_hook_2");
    }

    #[test]
    fn test_try_exec_ok() {
        let mut items = Vec::new();

        let res = hook::try_exec(my_mut_hook::with(&mut items));

        assert!(res.is_ok());
        assert_eq!(items.len(), 2);
        assert_eq!(items[0], "my_mut_hook_1");
        assert_eq!(items[1], "my_mut_hook_2");
    }

    #[hook]
    fn my_bad_mut_hook(output: &mut Vec<&str>) -> Result<(), String> {}

    #[hook(my_bad_mut_hook, 1)]
    fn my_bad_mut_hook_1(output: &mut Vec<&str>) -> Result<(), String> {
        output.push("my_bad_mut_hook_1");
        Err(String::from("my_bad_mut_hook_1"))
    }

    #[hook(my_bad_mut_hook, 2)]
    fn my_bad_mut_hook_2(output: &mut Vec<&str>) -> Result<(), String> {
        output.push("my_bad_mut_hook_2");
        Ok(())
    }

    #[test]
    fn test_try_exec_err() {
        let mut items = Vec::new();

        let res = hook::try_exec(my_bad_mut_hook::with(&mut items));

        assert!(res.is_err());
        assert_eq!(res, Err(String::from("my_bad_mut_hook_1")));
        assert_eq!(items.len(), 1);
        assert_eq!(items[0], "my_bad_mut_hook_1");
    }
}
