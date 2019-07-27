use crate::helpers::{qlc_command_with_fake_dir, qlc_command_with_fake_dir_and_schema};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::str::contains;

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
    cmd.assert().failure().stderr(contains("ParseError"));
}

#[test]
fn compile_with_non_schema_matching_graphql() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str("query Name { does_not_exist }")
        .unwrap();
    cmd.assert()
        .failure()
        .stderr(contains("UnknownField(\"Query\", \"does_not_exist\")"));
}
