use super::helpers::qlc_command_with_fake_dir;
use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use predicates::str::contains;

#[test]
fn schema_file_does_not_exist() {
    qlc_command_with_fake_dir()
        .0
        .assert()
        .stderr(contains("No such file or directory"))
        .failure();
}

#[test]
fn schema_file_does_not_exist_with_flag() {
    qlc_command_with_fake_dir()
        .0
        .arg("-s")
        .arg("not-a-real-file.json")
        .assert()
        .stderr(contains("No such file or directory"))
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
        .stderr(contains("JSONParseError"))
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
        .stderr(contains("missing field `data`"))
        .failure();
}
