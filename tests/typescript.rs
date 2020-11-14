use crate::helpers::basic_success_assert;

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
