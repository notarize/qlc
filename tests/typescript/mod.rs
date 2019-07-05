use crate::helpers::basic_success_assert;

mod enumeration;
mod interface;

#[test]
fn compile_simple_query() {
    basic_success_assert(
        "
query TestQuery {
  viewer {
    id
    me: user {
      id
    }
  }
}
    ",
        "TestQuery.ts",
        "
export interface TestQuery_viewer_me {
  id: string;
}

export interface TestQuery_viewer {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  me: TestQuery_viewer_me | null;
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
export interface myViewerFragment_user {
  id: string;
}

export interface myViewerFragment {
  id: string;
  /**
   * The user associated with the current viewer. Use this field to get info
   * about current viewer and access any records associated w/ their account.
   */
  user: myViewerFragment_user | null;
}
    ",
    );
}
