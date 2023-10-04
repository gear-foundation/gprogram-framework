#[test]
fn play() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/play.rs");
}

#[test]
fn sync_command_handlers_works() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/sync_command_handlers_works.rs");
}
