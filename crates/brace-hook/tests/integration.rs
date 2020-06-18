use brace_hook::{hook, register};

#[hook]
fn my_hook(input: &str) -> String {}

#[hook(my_hook)]
fn hook_1(input: &str) -> String {
    format!("hook_1: {}", input)
}

fn hook_2(input: &str) -> String {
    format!("hook_2: {}", input)
}

fn hook_3(input: &str) -> String {
    format!("hook_3: {}", input)
}

register!(my_hook, hook_2);
register!(my_hook, hook_3, 0);

#[test]
fn test_hook_registration() {
    let res: Vec<String> = my_hook::invoke("hello");

    assert_eq!(res.len(), 3);

    assert!(res.contains(&String::from("hook_1: hello")));
    assert!(res.contains(&String::from("hook_2: hello")));
    assert!(res.contains(&String::from("hook_3: hello")));
}

#[hook]
fn empty() {}

#[hook(empty)]
fn empty_1() {}

#[test]
fn test_hook_without_args() {
    assert_eq!(empty::invoke().len(), 1);
}

#[hook]
fn unused() {}

#[test]
fn test_hook_without_impls() {
    assert_eq!(unused::invoke().len(), 0);
}

#[hook]
fn weighted() -> &'static str {}

#[hook(weighted, 300)]
fn weighted_a() -> &'static str {
    "a"
}

#[hook(weighted, 0)]
fn weighted_b() -> &'static str {
    "b"
}

#[hook(weighted, 20)]
fn weighted_c() -> &'static str {
    "c"
}

#[hook(weighted, -50)]
fn weighted_d() -> &'static str {
    "d"
}

#[test]
fn test_hook_with_weights() {
    let res: Vec<&'static str> = weighted::invoke();

    assert_eq!(res.len(), 4);
    assert_eq!(res[0], "d");
    assert_eq!(res[1], "b");
    assert_eq!(res[2], "c");
    assert_eq!(res[3], "a");
}

#[hook]
fn mutate(items: &mut Vec<&str>) {}

#[hook(mutate, 1)]
fn mutate_1(items: &mut Vec<&str>) {
    items.push("mutate 1");
}

#[hook(mutate, 2)]
fn mutate_2(items: &mut Vec<&str>) {
    items.push("mutate 2");
}

#[hook(mutate, 3)]
fn mutate_3(items: &mut Vec<&str>) {
    items.push("mutate 3");
}

#[test]
fn test_hook_with_mutations() {
    let mut items = Vec::new();

    let res = mutate::invoke(&mut items);

    assert_eq!(res.len(), 3);
    assert_eq!(items.len(), 3);

    assert_eq!(items[0], "mutate 1");
    assert_eq!(items[1], "mutate 2");
    assert_eq!(items[2], "mutate 3");
}

mod custom {
    pub use brace_hook::*;
}

#[hook]
#[hook_attr(crate = custom)]
fn relocate() {}

#[hook(relocate, 1)]
#[hook_attr(crate = custom)]
fn relocated() {}

#[test]
fn test_crate_location() {
    assert_eq!(relocate::invoke().len(), 1);
}

mod nested {
    use super::hook;

    #[hook]
    pub fn visibility() {}

    #[hook(visibility, 1)]
    fn visible_1() {}

    #[hook(visibility, 1)]
    pub fn visible_2() {}
}

#[test]
fn test_hook_visibility() {
    assert_eq!(nested::visibility::invoke().len(), 2);
}

#[hook]
fn hook_with_default(a: &str, b: &str) -> String {
    format!("a: {}, b: {}", a, b)
}

#[test]
fn test_hook_with_default() {
    let res = hook_with_default::invoke("one", "two");

    assert_eq!(res.len(), 1);
    assert_eq!(res[0], "a: one, b: two");
}

#[hook]
fn hook_with_default_unused(a: &str, b: &str) -> String {
    format!("a: {}, b: {}", a, b)
}

#[hook(hook_with_default_unused)]
fn hook_with_default_unused_1(a: &str, b: &str) -> String {
    format!("b: {}, a: {}", b, a)
}

#[test]
fn test_hook_with_default_unused() {
    let res = hook_with_default_unused::invoke("one", "two");

    assert_eq!(res.len(), 1);
    assert_eq!(res[0], "b: two, a: one");
}
