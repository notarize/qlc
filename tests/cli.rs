use crate::helpers::cmd::TestCommandHarness;
use crate::helpers::stdout_predicates::{
    contains_graphql_file_error_without_location, contains_no_such_file_error,
};
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{contains, is_empty};

#[test]
fn run_with_empty_dir() {
    TestCommandHarness::default().run_for_success();
}

#[test]
fn run_without_schema_file() {
    TestCommandHarness::new()
        .run_for_failure()
        .stdout(contains("/schema.json`: No such file or directory"));
}

#[test]
fn compile_with_importing_query_instead_of_fragment() {
    let mut harness = TestCommandHarness::default();

    let assertion = contains(
        "= help: This document is not a fragment, and importing it is probably a mistake.",
    )
    .and(contains(
        "#import \"./imported_query.graphql\"\n  |         ^",
    ))
    .and(contains_graphql_file_error_without_location(
        harness.directory_path().join("main_query.graphql"),
    ));

    harness
        .with_fixture_directory("cli/compile_with_importing_query_instead_of_fragment")
        .run_for_failure()
        .stdout(assertion);
}

#[test]
fn compile_with_missing_fragment() {
    let mut harness = TestCommandHarness::default();
    let dir_path = harness.directory_path();
    let assertion = contains_no_such_file_error(dir_path.join("./not_here.graphql"))
        .and(contains("#import \"./not_here.graphql\"\n  |         ^"))
        .and(contains_graphql_file_error_without_location(
            dir_path.join("importing_missing_query.graphql"),
        ));
    harness
        .with_fixture_directory("cli/compile_with_missing_fragment")
        .run_for_failure()
        .stdout(assertion);
}

#[test]
fn run_with_unparseable_graphql() {
    let mut harness = TestCommandHarness::default();

    let assertion =
        contains("Parse error at 2:19").and(contains_graphql_file_error_without_location(
            harness.directory_path().join("unparseable.graphql"),
        ));

    harness
        .with_fixture_directory("cli/run_with_unparseable_graphql")
        .run_for_failure()
        .stdout(assertion);
}

#[test]
fn run_with_broken_config_file() {
    TestCommandHarness::default()
        .with_default_rc_file_contents("{ \"notValidJson: true }")
        .run_for_failure()
        .stdout(contains("program error: error in config file").and(contains(".qlcrc.json`")));
}

#[test]
fn run_with_config_file_missing_schema() {
    let mut harness = TestCommandHarness::default();

    let assertion =
        contains_no_such_file_error(harness.directory_path().join("not_default_schema.json"));

    harness
        .with_default_rc_file_contents("{ \"schemaFile\": \"not_default_schema.json\" }")
        .run_for_failure()
        .stdout(assertion);
}

#[test]
fn run_with_invalid_config_file_schema_and_valid_cli_override() {
    let mut harness = TestCommandHarness::default();
    let schema_file_path = harness.directory_path().join("schema.json");
    harness
        // put an invalid schema file in config file
        .with_default_rc_file_contents("{ \"schemaFile\": \"not_default_schema.json\" }")
        // but override with valid cli arg
        .with_arg("-s")
        .with_arg(schema_file_path)
        .run_for_success()
        .stdout(is_empty());
}

#[test]
fn run_with_invalid_cli_schema_arg() {
    let missing_schema_path = "not_a_real_file.json";
    TestCommandHarness::default()
        .with_arg("-s")
        .with_arg(missing_schema_path)
        .run_for_failure()
        .stdout(contains_no_such_file_error(missing_schema_path));
}

#[test]
fn run_with_invalid_schema_json_syntax() {
    TestCommandHarness::new()
        .with_default_schema_file_from_contents("{ \"notvalidJs: true ")
        .run_for_failure()
        .stdout(contains(
            "error: malformed schema: JSON parse error: EOF while parsing",
        ));
}

#[test]
fn run_with_wrong_shape_schema_json() {
    TestCommandHarness::new()
        // syntatically valid json but malformed shape
        .with_default_schema_file_from_contents("{ \"unexpected\": 3 }")
        .run_for_failure()
        .stdout(contains(
            "error: malformed schema: JSON parse error: missing field `data` at line 1",
        ));
}

#[test]
fn compile_with_use_show_deprecation_warnings() {
    let assertion = contains("warning: use of deprecated field `publicRSAKey` on type `User`")
        .and(contains("4 |     publicRSAKey\n  |     ^"));
    TestCommandHarness::default()
        .with_fixture_directory("cli/compile_with_use_show_deprecation_warnings")
        .with_arg("--show-deprecation-warnings")
        .run_for_success()
        .stdout(assertion);
}
