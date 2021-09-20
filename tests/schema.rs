use super::helpers::{
    contains_read_error, qlc_command_with_fake_dir, qlc_command_with_fake_dir_and_schema,
};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::str::{contains, is_empty};

#[test]
fn schema_file_does_not_exist() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir();
    cmd.assert()
        .stderr(is_empty())
        .stdout(contains_read_error(
            &temp_dir,
            "schema.json",
            "No such file or directory (os error 2)",
        ))
        .failure();
}

#[test]
fn schema_file_does_not_exist_with_flag() {
    qlc_command_with_fake_dir()
        .0
        .arg("-s")
        .arg("not-a-real-file.json")
        .assert()
        .stdout(contains(
            "error: could not read `not-a-real-file.json`: No such file or directory (os error 2)",
        ))
        .stderr(is_empty())
        .failure();
}

#[test]
fn schema_file_invalid_json_syntax() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir();
    let bad_syntax_file = temp_dir.child("schema.json");
    bad_syntax_file.write_str("t").unwrap();
    cmd.arg("-s")
        .arg(bad_syntax_file.path())
        .assert()
        .stdout(contains("error: malformed schema: JSON parse error: EOF while parsing a value at line 1 column 1"))
        .stderr(is_empty())
        .failure();
}

#[test]
fn schema_file_invalid_schema_json() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir();
    let unexpected_json_file = temp_dir.child("schema.json");
    unexpected_json_file
        .write_str("{\"unexpected\": 3}")
        .unwrap();
    cmd.arg("-s")
        .arg(unexpected_json_file.path())
        .assert()
        .stdout(contains(
            "error: malformed schema: JSON parse error: missing field `data` at line 1 column 17",
        ))
        .stderr(is_empty())
        .failure();
}

#[test]
fn use_of_deprecated_fields() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("deprecated.graphql")
        .write_str(
            r#"
query Deprecated($id: ID!) {
  node(id: $id) {
    ... on Organization {
      features
    }
  }
}"#,
        )
        .unwrap();
    cmd.arg("--show-deprecation-warnings")
        .assert()
        .stdout(contains(
            "warning: use of deprecated field `features` on type `Organization`",
        ))
        .stderr(is_empty())
        .success();
}
