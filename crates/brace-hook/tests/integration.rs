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

#[hook(weight = 1000)]
fn weighted() -> &'static str {
    "0"
}

#[hook(name = "weighted", weight = 300)]
fn weighted_a() -> &'static str {
    "a"
}

#[hook(name = "weighted", weight = 0)]
fn weighted_b() -> &'static str {
    "b"
}

#[hook(name = "weighted", weight = 20)]
fn weighted_c() -> &'static str {
    "c"
}

#[test]
fn test_static_discovery_weights() {
    let res: Vec<&'static str> = invoke_all("weighted", ()).unwrap();

    assert_eq!(res.len(), 4);
    assert_eq!(res[0], "b");
    assert_eq!(res[1], "c");
    assert_eq!(res[2], "a");
    assert_eq!(res[3], "0");
}
