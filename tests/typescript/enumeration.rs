use crate::helpers::cmd::TestCommandHarness;

#[test]
fn compile_with_global_types() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/enumeration/compile_with_global_types")
        .run_for_success();
}
