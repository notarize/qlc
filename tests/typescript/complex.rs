use crate::helpers::cmd::TestCommandHarness;

mod interface;
mod union;

#[test]
fn compile_deep_fragments() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/compile_deep_fragments")
        .run_for_success();
}

#[test]
fn compile_absolute_import_fragments() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/compile_absolute_import_fragments")
        .run_for_success();
}
