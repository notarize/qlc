use crate::helpers::cmd::TestCommandHarness;

mod complex;
mod enumeration;
mod field;
mod variable;

#[test]
fn compile_simple_query() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/compile_simple_query")
        .run_for_success();
}

#[test]
fn compile_simple_subscription() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/compile_simple_subscription")
        .run_for_success();
}

#[test]
fn compile_simple_mutation() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/compile_simple_mutation")
        .run_for_success();
}

#[test]
fn compile_simple_fragment() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/compile_simple_fragment")
        .run_for_success();
}

#[test]
fn compile_typename() {
    TestCommandHarness::default()
        .with_fixture_directory("typescript/compile_typename")
        .run_for_success();
}

#[test]
fn compile_with_all_module_config() {
    TestCommandHarness::default()
        .with_arg("--global-types-module-name=@web/graphql_globals")
        .with_arg("--root-dir-import-prefix=@web/")
        .with_arg("--typed-graphql-documentnode-module-name=@web/typed_node")
        .with_fixture_directory("typescript/compile_with_all_module_config")
        .run_for_success();
}
