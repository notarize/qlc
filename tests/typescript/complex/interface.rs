use crate::helpers::basic_success_assert;

#[test]
fn compile_interface_concrete_only_query() {
    basic_success_assert(
        "
query TestQuery {
  meeting: node {
    id
  }
}
        ",
        "TestQuery.ts",
        "
export interface TestQuery_meeting_Node {
  /**
   * ID of the object.
   */
  id: string;
}

export type TestQuery_meeting = TestQuery_meeting_Node;

export interface TestQuery {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
}
        ",
    );
}

#[test]
fn compile_interface_no_concrete_query() {
    basic_success_assert(
        "
query TestQuery {
  meeting: node {
    ... on Meeting {
      canJoin: can_join
      can_complete
    }
  }
}
        ",
        "TestQuery.ts",
        "
export interface TestQuery_meeting_Meeting {
  /**
   * Whether or not the meeting is ongoing and the user can join it
   */
  canJoin: boolean;
  can_complete: boolean;
}

export type TestQuery_meeting = TestQuery_meeting_Meeting;

export interface TestQuery {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
}
        ",
    );
}

#[test]
fn compile_interface_both_concrete_and_implementing() {
    basic_success_assert(
        "
query TestQuery {
  meeting: node {
    id
    ... on Meeting {
      canComplete: can_complete
    }
  }
}
        ",
        "TestQuery.ts",
        "
export interface TestQuery_meeting_Meeting {
  canComplete: boolean;
}

export interface TestQuery_meeting_Node {
  /**
   * ID of the object.
   */
  id: string;
}

export type TestQuery_meeting = (TestQuery_meeting_Meeting) & TestQuery_meeting_Node;

export interface TestQuery {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
}
       ",
    );
}
