use crate::helpers::basic_success_assert;

#[test]
fn compile_interface_with_typename() {
    basic_success_assert(
        "
query TestQuery {
  meeting: node {
    __typename
    as: __typename
    ... on Meeting {
      __typename
      another: __typename
      can_complete
    }
  }
}
        ",
        "TestQuery.ts",
        r#"
export type TestQuery_meeting_Meeting = {
  __typename: "Meeting";
  another: "Meeting";
  as: "Meeting";
  can_complete: boolean;
};

export type TestQuery_meeting_$$other = {
  __typename: "AdminProfile" | "AnnotationDesignation" | "BaseTransaction" | "CheckmarkAnnotation" | "Customer" | "Document" | "DocumentBundle" | "ImageAnnotation" | "Lender" | "MeetingRequest" | "MortgageBorrower" | "Notary" | "NotaryProfile" | "Organization" | "OrganizationDocumentTemplate" | "OrganizationMembership" | "OrganizationTransaction" | "OrganizationUser" | "RecordingLocation" | "SignerIdentity" | "TextAnnotation" | "TitleAgency" | "UsState" | "User" | "VectorGraphicAnnotation" | "Viewer" | "WhiteboxAnnotation" | "WitnessProfile";
  as: "AdminProfile" | "AnnotationDesignation" | "BaseTransaction" | "CheckmarkAnnotation" | "Customer" | "Document" | "DocumentBundle" | "ImageAnnotation" | "Lender" | "MeetingRequest" | "MortgageBorrower" | "Notary" | "NotaryProfile" | "Organization" | "OrganizationDocumentTemplate" | "OrganizationMembership" | "OrganizationTransaction" | "OrganizationUser" | "RecordingLocation" | "SignerIdentity" | "TextAnnotation" | "TitleAgency" | "UsState" | "User" | "VectorGraphicAnnotation" | "Viewer" | "WhiteboxAnnotation" | "WitnessProfile";
};

export type TestQuery_meeting = TestQuery_meeting_Meeting | TestQuery_meeting_$$other;

export type TestQuery = {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
};
        "#,
    );
}

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
export type TestQuery_meeting = {
  /**
   * ID of the object.
   */
  id: string;
};

export type TestQuery = {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
};
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
export type TestQuery_meeting_Meeting = {
  /**
   * Whether or not the meeting is ongoing and the user can join it
   */
  canJoin: boolean;
  can_complete: boolean;
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
export type TestQuery_meeting_Meeting = {
  canComplete: boolean;
  /**
   * ID of the object.
   */
  id: string;
};

export type TestQuery_meeting_$$other = {
  /**
   * ID of the object.
   */
  id: string;
};

export type TestQuery_meeting = TestQuery_meeting_Meeting | TestQuery_meeting_$$other;

export type TestQuery = {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
};
       ",
    );
}
