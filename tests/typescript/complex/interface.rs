use crate::helpers::cmd::TestCommandHarness;

#[test]
fn compile_interface_with_typename() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/interface/compile_interface_with_typename")
        .run_for_success();
}

#[test]
fn compile_interface_abstract_only() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/interface/compile_interface_abstract_only")
        .run_for_success();
}

#[test]
fn compile_interface_concrete_only() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/interface/compile_interface_concrete_only")
        .run_for_success();
}

#[test]
fn compile_interface_both_concrete_and_abstract() {
    TestCommandHarness::default()
        .with_fixture_directory(
            "typescript/complex/interface/compile_interface_both_concrete_and_abstract",
        )
        .run_for_success();
}
