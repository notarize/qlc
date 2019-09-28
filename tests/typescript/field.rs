use crate::helpers::{
    assert_generated, basic_success_assert, qlc_command_with_fake_dir_and_schema,
};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;

#[test]
fn compile_custom_scalar_any() {
    basic_success_assert(
        "
query TestQuery {
  viewer {
    user {
      id
      created_at
    }
  }
}
    ",
        "TestQuery.ts",
        "
export interface TestQuery_viewer_user {
  created_at: any | null;
  id: string;
}

export interface TestQuery_viewer {
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
}

export interface TestQuery {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
}
    ",
    );
}

#[test]
fn compile_custom_scalar_with_names() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    cmd.arg("--use-custom-scalars");
    temp_dir
        .child("file.graphql")
        .write_str(
            "
query TestQuery {
  viewer {
    user {
      id
      created_at
    }
  }
}
    ",
        )
        .unwrap();
    cmd.assert().success();
    assert_generated(
        &temp_dir,
        "TestQuery.ts",
        "
export interface TestQuery_viewer_user {
  created_at: Date | null;
  id: string;
}

export interface TestQuery_viewer {
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
}

export interface TestQuery {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
}
    ",
    );
}
