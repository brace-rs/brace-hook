use brace_hook::{hook, invoke_all, register};

#[hook]
fn my_hook(input: &str) -> String {
    format!("zero: {}", input)
}

#[hook(name = "my_hook")]
fn hook_1(input: &str) -> String {
    format!("one: {}", input)
}

fn hook_2(input: &str) -> String {
    format!("two: {}", input)
}

register!("my_hook", hook_2);

#[test]
fn test_static_discovery() {
    let res: Vec<String> = invoke_all("my_hook", ("hello",)).unwrap();

    assert_eq!(res.len(), 3);

    assert!(res.contains(&String::from("zero: hello")));
    assert!(res.contains(&String::from("one: hello")));
    assert!(res.contains(&String::from("two: hello")));
}
