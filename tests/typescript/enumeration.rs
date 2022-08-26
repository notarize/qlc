use crate::helpers::basic_success_with_global_types_assert;

#[test]
fn compile_with_global_types() {
    basic_success_with_global_types_assert(
        "
query TestQuery {
  meeting: node {
    ... on Meeting {
       endedState: ended_state
    }
  }
}
        ",
        "TestQuery.ts",
        r#"
import type { MeetingEndedState } from "__generated__/globalTypes";

export type TestQuery_meeting_Meeting = {
  endedState: MeetingEndedState;
};

export type TestQuery_meeting_$$other = {

};

export type TestQuery_meeting = TestQuery_meeting_Meeting | TestQuery_meeting_$$other;

export type TestQuery = {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
};
        "#,
        r#"
/**
 * Describes the state of the meeting at the time it completed
 */
export enum MeetingEndedState {
  NOT_COMPLETED = "NOT_COMPLETED",
  KILLED = "KILLED",
  CUSTOMER_CANCELLED = "CUSTOMER_CANCELLED",
  NOTARY_CANCELLED_WITH_CHARGE = "NOTARY_CANCELLED_WITH_CHARGE",
  NOTARY_CANCELLED_NO_CHARGE = "NOTARY_CANCELLED_NO_CHARGE",
  COMPLETED = "COMPLETED",
}
        "#,
    );
}
