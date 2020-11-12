use crate::helpers::{
    contains_graphql_filename, contains_read_error, qlc_command_with_fake_dir,
    qlc_command_with_fake_dir_and_schema,
};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::{contains, is_empty};

#[test]
fn compile_without_schema_file() {
    let (mut cmd, _) = qlc_command_with_fake_dir();
    cmd.assert().failure();
}

#[test]
fn compile_with_importing_query_instead_of_fragment() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(
            r#"#import "./other_query.graphql"
query OperationName { node { id } }
"#,
        )
        .unwrap();
    temp_dir
        .child("other_query.graphql")
        .write_str("query OtherQuery { node { id } }")
        .unwrap();
    let assertion = contains(
        "= help: This document is not a fragment, and importing it is probably a mistake.",
    )
    .and(contains("#import \"./other_query.graphql\"\n  |         ^"))
    .and(contains_graphql_filename(&temp_dir, "file.graphql", None));
    cmd.assert().failure().stdout(assertion).stderr(is_empty());
}

#[test]
fn compile_with_missing_fragment() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(
            r#"#import "./not_here.graphql"
query OperationName { node { id } }
"#,
        )
        .unwrap();
    let assertion = contains_read_error(
        &temp_dir,
        "./not_here.graphql",
        "No such file or directory (os error 2)",
    )
    .and(contains("#import \"./not_here.graphql\"\n  |         ^"))
    .and(contains_graphql_filename(&temp_dir, "file.graphql", None));
    cmd.assert().failure().stdout(assertion).stderr(is_empty());
}

#[test]
fn compile_with_unparseable_graphql() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str("query Name {{ thing }")
        .unwrap();
    let assertion = contains("Parse error at 1:13").and(contains_graphql_filename(
        &temp_dir,
        "file.graphql",
        None,
    ));
    cmd.assert().failure().stdout(assertion).stderr(is_empty());
}

#[test]
fn compile_with_narrowing() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("meeting_fragment.graphql")
        .write_str("fragment externalSpreadOnMeeting on Meeting { id }")
        .unwrap();
    temp_dir
        .child("file.graphql")
        .write_str(
            r#"
#import "./meeting_fragment.graphql"
query Narrowing {
  viewer {
    id
    ...externalSpreadOnMeeting
    ... on User {
      id
    }
  }
}
"#,
        )
        .unwrap();
    let assertion_external = contains("= help: The parent types of this spread are limited to `Viewer`, making spreading `Meeting` uneeded.")
      .and(contains("6 |     ...externalSpreadOnMeeting\n  |        ^"))
      .and(contains_graphql_filename(
            &temp_dir,
            "file.graphql",
            Some((6, 8)),
        ));
    let assertion_inline = contains("= help: The parent types of this spread are limited to `Viewer`, making spreading `User` uneeded.")
      .and(contains("7 |     ... on User {\n  |         ^"))
      .and(contains_graphql_filename(
            &temp_dir,
            "file.graphql",
            Some((7, 9)),
        ));
    cmd.assert()
        .success()
        .stdout(assertion_inline.and(assertion_external))
        .stderr(is_empty());
}

#[test]
fn compile_with_non_schema_matching_graphql() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(
            r#"query QueryOperation {
  doesNotExist
  alsoIsNot: nonExist
}"#,
        )
        .unwrap();
    let assertion = contains("= help: Check the fields of `Query`.")
        .and(contains("2 |   doesNotExist\n  |   ^"))
        .and(contains("3 |   alsoIsNot: nonExist\n  |   ^"))
        .and(contains_graphql_filename(
            &temp_dir,
            "file.graphql",
            Some((3, 3)),
        ))
        .and(contains_graphql_filename(
            &temp_dir,
            "file.graphql",
            Some((2, 3)),
        ));
    cmd.assert().failure().stdout(assertion).stderr(is_empty());
}
