use crate::helpers::{assert_generated, diff, qlc_command_with_fake_dir_and_schema};
use assert_cmd::prelude::*;
use assert_fs::prelude::*;

mod interface;
mod union;

#[test]
fn compile_deep_fragments() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    user {
      id
      firstName: first_name
    }
    ... on Viewer {
      id
      id2: id
      ... on Viewer {
        id3: id
      }
      ...testFragment
    }
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            "
fragment testFragment on Viewer {
  user {
    id
    lastName: last_name
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
  firstName: string | null;
  id: string;
  lastName: string | null;
};

export type TestQuery_viewer = {
  id: string;
  id2: string;
  id3: string;
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
fn compile_single_fragment() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    ...testFragment
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            "
fragment testFragment on Viewer {
  user { id }
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
  id: string;
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
    ",
    );
}

#[test]
fn compile_absolute_import_fragments() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "user/userfragment_one.graphql"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    ...testFragment
    user {
      ...absoluteUserFragmentOne
    }
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            r#"
#import "user/userfragment_one.graphql"
#import "user/userfragment_two.graphql"

fragment testFragment on Viewer {
  user {
    id
    roles
    ...absoluteUserFragmentOne
    ...absoluteUserFragmentTwo
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("user/userfragment_one.graphql")
        .write_str(
            r#"
fragment absoluteUserFragmentOne on User {
  id
  as: __typename
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("user/userfragment_two.graphql")
        .write_str(
            r#"
fragment absoluteUserFragmentTwo on User {
  id
  createdAt: created_at
  customerProfile: customer_profile {
    id
  }
}
        "#,
        )
        .unwrap();

    cmd.assert().success();

    assert_generated(
        &temp_dir,
        "TestQuery.ts",
        r#"
import type { UserRole } from "__generated__/globalTypes";

export type TestQuery_viewer_user_customerProfile = {
  id: string;
};

export type TestQuery_viewer_user = {
  as: "User";
  createdAt: any | null;
  customerProfile: TestQuery_viewer_user_customerProfile | null;
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
    );

    assert_generated(
        &temp_dir,
        "globalTypes.ts",
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
    );

    let frag_one = temp_dir
        .child("user")
        .child("__generated__")
        .child("absoluteUserFragmentOne.ts");
    frag_one.assert(diff(
        r#"
export type absoluteUserFragmentOne = {
  as: "User";
  id: string;
};
    "#,
    ));

    let frag_two = temp_dir
        .child("user")
        .child("__generated__")
        .child("absoluteUserFragmentTwo.ts");
    frag_two.assert(diff(
        r#"
export type absoluteUserFragmentTwo_customerProfile = {
  id: string;
};

export type absoluteUserFragmentTwo = {
  createdAt: any | null;
  customerProfile: absoluteUserFragmentTwo_customerProfile | null;
  id: string;
};
    "#,
    ));
}

#[test]
fn compile_recursive_fragment_with_global() {
    let (mut cmd, temp_dir) = qlc_command_with_fake_dir_and_schema();
    temp_dir
        .child("main.graphql")
        .write_str(
            r#"
#import "./testfragment.graphql"

query TestQuery {
  viewer {
    id
    ...testFragment
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("testfragment.graphql")
        .write_str(
            r#"
#import "./userFragmentOne.graphql"
#import "./test/userFragmentTwo.graphql"

fragment testFragment on Viewer {
  user {
    id
    roles
    ...userFragmentOne
    ...userFragmentTwo
  }
}
        "#,
        )
        .unwrap();

    temp_dir
        .child("userFragmentOne.graphql")
        .write_str(
            "
fragment userFragmentOne on User {
  systemId: system_id
}
        ",
        )
        .unwrap();

    temp_dir
        .child("test")
        .child("userFragmentTwo.graphql")
        .write_str(
            "
fragment userFragmentTwo on User {
  featureList: feature_list
  singleUse: single_use
  scheduled_tiers {
    active
    endAt: end_at
  }
}
        ",
        )
        .unwrap();

    cmd.assert().success();

    assert_generated(
        &temp_dir,
        "TestQuery.ts",
        r#"
import type { Feature, UserRole } from "__generated__/globalTypes";

export type TestQuery_viewer_user_scheduled_tiers = {
  /**
   * Flag indicating if currently running and active schedule
   */
  active: boolean;
  /**
   * Tier schedule end time or indefinite
   */
  endAt: any | null;
};

export type TestQuery_viewer_user = {
  /**
   * An user's active features and features inherited from tier
   */
  featureList: Feature[];
  id: string;
  roles: (UserRole | null)[] | null;
  /**
   * A user's scheduled tiers including historical, current and future
   */
  scheduled_tiers: TestQuery_viewer_user_scheduled_tiers[];
  /**
   * Whether or not user is being used for single transaction
   */
  singleUse: boolean;
  systemId: number | null;
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
    );
    assert_generated(
        &temp_dir,
        "globalTypes.ts",
        r#"
/**
 * The possible standalone features and features inherited from tier for an orgnization
 */
export enum Feature {
  PARTNER_API = "PARTNER_API",
  EMPLOYEES = "EMPLOYEES",
  CUSTOM_LOGO = "CUSTOM_LOGO",
  DOCUMENT_TEMPLATES = "DOCUMENT_TEMPLATES",
  TRANSACTION_TEMPLATES = "TRANSACTION_TEMPLATES",
  TRANSACTION_ANALYTICS = "TRANSACTION_ANALYTICS",
  TRANSACTION_FILTERING = "TRANSACTION_FILTERING",
  ADVANCED_TRANSACTION_DETAILS = "ADVANCED_TRANSACTION_DETAILS",
  ADVANCED_TRANSACTION_CREATION = "ADVANCED_TRANSACTION_CREATION",
  CUSTOM_EMAILS = "CUSTOM_EMAILS",
  SMS_INVITATION = "SMS_INVITATION",
  ESIGN = "ESIGN",
  TRANSACTION_RECORD_FULL_ACCESS = "TRANSACTION_RECORD_FULL_ACCESS",
  BATCH_PAYMENT = "BATCH_PAYMENT",
}

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
    );
}
