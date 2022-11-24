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
export type TestQuery_viewer_user = {
  created_at: any | null;
  id: string;
};

export type TestQuery_viewer = {
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
    ",
    );
}

#[test]
fn compile_custom_scalar_with_default_names() {
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
export type TestQuery_viewer_user = {
  created_at: Date | null;
  id: string;
};

export type TestQuery_viewer = {
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
        ",
    );
}

#[test]
fn compile_custom_scalar_with_prefix() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    cmd.arg("--use-custom-scalars");
    cmd.arg("--custom-scalar-prefix=Prefix");
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
export type TestQuery_viewer_user = {
  created_at: PrefixDate | null;
  id: string;
};

export type TestQuery_viewer = {
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
        ",
    );
}

#[test]
fn compile_deprecated_field() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("file.graphql")
        .write_str(
            "
query Deprecated($orgId: ID!) {
  viewer {
    user {
      tier
    }
  }
  org: node(id: $orgId) {
    ... on Organization {
      features
    }
  }
}
            ",
        )
        .unwrap();
    cmd.assert().success();
    assert_generated(
        &temp_dir,
        "Deprecated.ts",
        r#"
import type { OrganizationAvailableFeatures, Tier } from "__generated__/globalTypes";

export type Deprecated_org_Organization = {
  /**
   * Features available for the organization to use (from LaunchDarkly)
   * @deprecated
   */
  features: (OrganizationAvailableFeatures | null)[] | null;
};

export type Deprecated_org_$$other = {

};

export type Deprecated_org = Deprecated_org_Organization | Deprecated_org_$$other;

export type Deprecated_viewer_user = {
  /**
   * @deprecated
   */
  tier: Tier;
};

export type Deprecated_viewer = {
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: Deprecated_viewer_user | null;
};

export type Deprecated = {
  /**
   * Fetches an object given its ID.
   */
  org: Deprecated_org | null;
  /**
   * Access to fields relevant to a consumer of the application
   */
  viewer: Deprecated_viewer | null;
};

export type DeprecatedVariables = {
  orgId: string;
};
        "#,
    );
}
