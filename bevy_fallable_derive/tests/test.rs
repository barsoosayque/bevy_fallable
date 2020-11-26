#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/fallable.rs");
    t.pass("tests/ui/fallable_keep.rs");
}
