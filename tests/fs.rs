use super::helpers::qlc_command_with_fake_dir_and_schema;
use assert_cmd::prelude::*;

#[test]
fn compile_empty_dir() {
    qlc_command_with_fake_dir_and_schema().0.assert().success();
}
