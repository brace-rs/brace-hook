use brace_hook::{invoke_all, register};

fn hook_1(input: &str) -> String {
    format!("one: {}", input)
}

fn hook_2(input: &str) -> String {
    format!("two: {}", input)
}

register!("hook", hook_1);
register!("hook", hook_2);

#[test]
fn test_static_discovery() {
    let res: Vec<String> = invoke_all("hook", ("hello",)).unwrap();

    assert_eq!(res.len(), 2);

    assert!(res.contains(&String::from("one: hello")));
    assert!(res.contains(&String::from("two: hello")));
}
