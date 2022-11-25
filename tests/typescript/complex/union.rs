use crate::helpers::cmd::TestCommandHarness;

#[test]
fn compile_union_with_concrete_only() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/union/compile_union_with_concrete_only")
        .run_for_success();
}

#[test]
fn compile_union_with_typenames() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/union/compile_union_with_typenames")
        .run_for_success();
}

#[test]
fn compile_union_with_abstract_only() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/union/compile_union_with_abstract_only")
        .run_for_success();
}

#[test]
fn compile_union_with_abstract_and_concrete_types() {
    TestCommandHarness::default()
        .with_fixture_directory(
            "typescript/complex/union/compile_union_with_abstract_and_concrete_types",
        )
        .run_for_success();
}

#[test]
fn compile_union_with_fragments() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/complex/union/compile_union_with_fragments")
        .run_for_success();
}
