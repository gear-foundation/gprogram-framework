#[test]
fn sync_command_handlers_work() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/sync_command_handlers_work.rs");
}

#[test]
fn sync_query_handlers_work() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/sync_query_handlers_work.rs");
}

#[test]
fn gprogram_works() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/gprogram_works.rs");
}

#[test]
fn no_command_handlers() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/no_command_handlers.rs");
}

#[test]
fn no_query_handlers() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/no_query_handlers.rs");
}

#[test]
fn async_command_handlers_not_implemented() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/async_command_handlers_not_implemented.rs");
}

#[test]
fn async_query_handlers_not_supported() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/async_query_handlers_not_supported.rs");
}

#[test]
fn command_handlers_must_be_inside_gprogram_1() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/command_handlers_must_be_inside_gprogram_1.rs");
}

#[test]
fn command_handlers_must_be_inside_gprogram_2() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/command_handlers_must_be_inside_gprogram_2.rs");
}

#[test]
fn query_handlers_must_be_inside_gprogram_1() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/query_handlers_must_be_inside_gprogram_1.rs");
}

#[test]
fn query_handlers_must_be_inside_gprogram_2() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/query_handlers_must_be_inside_gprogram_2.rs");
}

#[test]
fn gprogram_must_be_inline() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/gprogram_must_be_inline.rs");
}

#[test]
fn single_command_handlers_block() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/single_command_handlers_block.rs");
}

#[test]
fn single_query_handlers_block() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/single_query_handlers_block.rs");
}

#[test]
fn command_handler_returns_result() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/command_handler_returns_result.rs");
}

#[test]
fn query_handler_returns_result() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/query_handler_returns_result.rs");
}
