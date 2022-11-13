use crate::helpers::{basic_success_assert, diff, qlc_command_with_fake_dir_and_schema};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;

mod complex;
mod enumeration;
mod field;
mod variable;

#[test]
fn compile_simple_query() {
    basic_success_assert(
        "
query TestQuery {
  viewer {
    me: user {
      id
    }
    id
  }
}
    ",
        "TestQuery.ts",
        "
export type TestQuery_viewer_me = {
  id: string;
};

export type TestQuery_viewer = {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  me: TestQuery_viewer_me | null;
};

export type TestQuery = {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
};
    ",
    );
}

#[test]
fn compile_simple_subscription() {
    basic_success_assert(
        "
subscription TestSubscription {
  viewer {
    me: user {
      id
    }
    id
  }
}
    ",
        "TestSubscription.ts",
        "
export type TestSubscription_viewer_me = {
  id: string;
};

export type TestSubscription_viewer = {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  me: TestSubscription_viewer_me | null;
};

export type TestSubscription = {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestSubscription_viewer | null;
};
    ",
    );
}

#[test]
fn compile_typename() {
    basic_success_assert(
        "
query TestQuery {
  viewer {
    as: __typename
    user {
      __typename
      id
    }
  }
}
    ",
        "TestQuery.ts",
        r#"
export type TestQuery_viewer_user = {
  __typename: "User";
  id: string;
};

export type TestQuery_viewer = {
  as: "Viewer";
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
};

export type TestQuery = {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
};
    "#,
    );
}

#[test]
fn compile_simple_fragment() {
    basic_success_assert(
        "
fragment myViewerFragment on Viewer {
  id
  user { id }
}
    ",
        "myViewerFragment.ts",
        "
export type myViewerFragment_user = {
  id: string;
};

export type myViewerFragment = {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: myViewerFragment_user | null;
};
    ",
    );
}

#[test]
fn compile_simple_mutation() {
    basic_success_assert(
        "
mutation CreateWitness {
  createWitness {
    meeting {
      id
    }
  }
}
    ",
        "CreateWitness.ts",
        "
export type CreateWitness_createWitness_meeting = {
  id: string;
};

export type CreateWitness_createWitness = {
  meeting: CreateWitness_createWitness_meeting | null;
};

export type CreateWitness = {
  /**
   * Creates a witness
   */
  createWitness: CreateWitness_createWitness | null;
};
    ",
    );
}

#[test]
fn compile_with_all_module_config() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    cmd.arg("--global-types-module-name=global-types");
    cmd.arg("--root-dir-import-prefix=~/");
    cmd.arg("--generated-module-name=gen-me");
    temp_dir
        .child("root_fragment.graphql")
        .write_str(
            "
fragment RootCheck on Viewer {
  id
}
            ",
        )
        .unwrap();
    temp_dir
        .child("lower/testing.graphql")
        .write_str(
            "
fragment LowerTesting on Viewer {
  user {
    id
    email
  }
}
            ",
        )
        .unwrap();
    temp_dir
        .child("file.graphql")
        .write_str(
            r#"
#import "./lower/testing.graphql"
#import "~/root_fragment.graphql"

query TestQuery {
  viewer {
    user {
      roles
    }
    ...RootCheck
    ...LowerTesting
  }
}
            "#,
        )
        .unwrap();

    cmd.assert().success();
    let generated_dir = temp_dir.child("gen-me");
    generated_dir.child("TestQuery.ts").assert(diff(
        r#"
import type { UserRole } from "~/gen-me/global-types";

export type TestQuery_viewer_user = {
  /**
   * Email address of the user; only available to current user and admins.
   */
  email: string | null;
  id: string;
  roles: (UserRole | null)[] | null;
};

export type TestQuery_viewer = {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: TestQuery_viewer_user | null;
};

export type TestQuery = {
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: TestQuery_viewer | null;
};
        "#,
    ));
    generated_dir.child("global-types.ts").assert(diff(
        r#"
/**
 * Describes a user's role within the system
 */
export enum UserRole {
  ADMIN = "ADMIN",
  NOTARY = "NOTARY",
  CUSTOMER = "CUSTOMER",
  WITNESS = "WITNESS",
  ORGANIZATION_MEMBER = "ORGANIZATION_MEMBER",
  ORGANIZATION_MEMBER_OWNER = "ORGANIZATION_MEMBER_OWNER",
  ORGANIZATION_MEMBER_ADMIN = "ORGANIZATION_MEMBER_ADMIN",
  ORGANIZATION_MEMBER_EMPLOYEE = "ORGANIZATION_MEMBER_EMPLOYEE",
  ORGANIZATION_MEMBER_PARTNER = "ORGANIZATION_MEMBER_PARTNER",
  ORGANIZATION_MEMBER_NOTARIZE_CLOSING_OPS = "ORGANIZATION_MEMBER_NOTARIZE_CLOSING_OPS",
}
        "#,
    ));
}
