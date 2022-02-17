use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use predicates::str::contains;
use std::path::Path;
use std::process::{Command, Stdio};

fn add_no_capture(mut cmd: Command) -> Command {
    if std::env::args().any(|arg| arg == "--nocapture") {
        cmd.stdin(Stdio::inherit());
        cmd.stderr(Stdio::inherit());
    }
    cmd
}

pub fn contains_read_error(
    temp_dir: &TempDir,
    filename: &str,
    error_str: &str,
) -> impl predicates::Predicate<str> {
    contains(format!(
        "error: could not read `{}`: {error_str}",
        temp_dir.path().join(filename).display(),
    ))
}

pub fn contains_graphql_filename(
    temp_dir: &TempDir,
    filename: &str,
    position: Option<(usize, usize)>,
) -> impl predicates::Predicate<str> {
    let location = match position {
        Some((line, col)) => format!(":{line}:{col}"),
        None => "".to_string(),
    };
    contains(format!(
        "--> {}{location}",
        temp_dir.path().join(filename).display(),
    ))
}

pub fn qlc_command_with_fake_dir() -> (Command, TempDir) {
    let mut cmd = Command::cargo_bin("qlc").unwrap();
    let temp_dir = assert_fs::TempDir::new().unwrap();
    cmd.arg(temp_dir.path());
    cmd.arg("--num-threads=2");
    (cmd, temp_dir)
}

pub fn qlc_command_with_fake_dir_and_schema() -> (Command, TempDir) {
    let (cmd, temp_dir) = qlc_command_with_fake_dir();
    let schema_file_copy = Path::new("tests/schema.json");
    temp_dir
        .child("schema.json")
        .write_file(schema_file_copy)
        .unwrap();
    (cmd, temp_dir)
}

/// Diff predicate that ignores whitespace before and after the desired string
pub fn diff(orig: &'static str) -> predicates::str::DifferencePredicate {
    predicates::str::diff(format!("/* tslint:disable */\n/* eslint-disable */\n// This file was automatically generated and should not be edited.\n\n{}", orig.trim()))
}

pub fn assert_generated(dir: &TempDir, expected_file_name: &str, expected_content: &'static str) {
    let output = dir.child("__generated__").child(expected_file_name);
    output.assert(diff(expected_content));
}

/// The basic outline of a succesful compile:
///  * Make a fake dir
///  * Shove in the schema
///  * Write in a single file
///  * Expect a compiled output
pub fn basic_success_assert(
    graphql_content: &str,
    expected_file_name: &str,
    expected_content: &'static str,
) {
    let (cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(graphql_content)
        .unwrap();
    let mut cmd = add_no_capture(cmd);
    cmd.assert().success();
    assert_generated(&temp_dir, expected_file_name, expected_content);
}

/// Same as `basic_success_assert` but asserts a global types file as well
pub fn basic_success_with_global_types_assert(
    graphql_content: &str,
    expected_file_name: &str,
    expected_content: &'static str,
    expected_global_types_content: &'static str,
) {
    let (cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(graphql_content)
        .unwrap();
    let mut cmd = add_no_capture(cmd);
    cmd.assert().success();
    assert_generated(&temp_dir, expected_file_name, expected_content);
    assert_generated(&temp_dir, "globalTypes.ts", expected_global_types_content);
}
