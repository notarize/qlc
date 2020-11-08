use crate::helpers::{
    contains_graphql_filename, qlc_command_with_fake_dir, qlc_command_with_fake_dir_and_schema,
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
fn compile_with_bad_graphql() {
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
