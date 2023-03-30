use crate::helpers::cmd::TestCommandHarness;
use crate::helpers::stdout_predicates::contains_graphql_file_error_with_location;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

#[test]
fn compile_with_non_schema_matching_graphql() {
    let mut harness = TestCommandHarness::default();
    let bad_query_path = harness.directory_path().join("bad_schema_query.graphql");

    let assertion = contains("= help: Check the fields of `Query`.")
        .and(contains("2 |   doesNotExist\n  |   ^"))
        .and(contains("3 |   alsoIsNot: nonExistent\n  |   ^"))
        .and(contains_graphql_file_error_with_location(
            &bad_query_path,
            (3, 3),
        ))
        .and(contains_graphql_file_error_with_location(
            &bad_query_path,
            (2, 3),
        ));

    harness
        .with_fixture_directory("schema/compile_with_non_schema_matching_graphql")
        .run_for_failure()
        .stdout(assertion);
}

#[test]
fn compile_with_narrowing() {
    let mut harness = TestCommandHarness::default();
    let narrowing_query_path = harness.directory_path().join("narrow_query.graphql");

    let assertion_external = contains("= help: The parent types of this spread are limited to `User`, making spreading `Host` extraneous.")
      .and(contains("6 |     ...SpreadOnHost\n  |        ^"))
      .and(contains_graphql_file_error_with_location(
            &narrowing_query_path,
            (6, 8),
        ));
    let assertion_inline = contains("= help: The parent types of this spread are limited to `User`, making spreading `Network` extraneous.")
      .and(contains("7 |     ... on Network {\n  |         ^"))
      .and(contains_graphql_file_error_with_location(
            &narrowing_query_path,
            (7, 9),
        ));

    harness
        .with_fixture_directory("cli/compile_with_narrowing")
        .run_for_success()
        .stdout(assertion_inline.and(assertion_external));
}
