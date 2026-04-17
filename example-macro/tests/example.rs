#[test]
fn add7() {
    assert_eq!(example_macro::add7!(3), 10);
}

example_macro::awesome_type!();
