use crate::helpers::cmd::TestCommandHarness;

#[test]
fn compile_query_with_variables() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/variable/compile_query_with_variables")
        .run_for_success();
}

#[test]
fn compile_mutation_with_variables() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/variable/compile_mutation_with_variables")
        .run_for_success();
}

#[test]
fn compile_mutation_with_inputs_including_lists() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/variable/compile_mutation_with_inputs_including_lists")
        .run_for_success();
}
