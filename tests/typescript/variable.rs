use crate::helpers::{basic_success_assert, basic_success_with_global_types_assert};

#[test]
fn compile_query_with_variables() {
    basic_success_assert(
        "
query TestQuery($meetingId: ID!) {
  meeting: node(id: $meetingId) {
    id
  }
}
        ",
        "TestQuery.ts",
        r#"
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

export type TestQueryVariables = {
  meetingId: string;
};
        "#,
    );
}

#[test]
fn compile_mutation_with_list_recursive_inputs() {
    basic_success_with_global_types_assert(
        "
mutation CatDocs($input: CategorizeDocumentsInput!) {
  categorizeDocuments(input: $input) {
    success
  }
}
        ",
        "CatDocs.ts",
        r#"
import type { CategorizeDocumentsInput } from "__generated__/globalTypes";

export type CatDocs_categorizeDocuments = {
  success: boolean;
};

export type CatDocs = {
  categorizeDocuments: CatDocs_categorizeDocuments | null;
};

export type CatDocsVariables = {
  input: CategorizeDocumentsInput;
};
        "#,
        r#"

export type CategorizeDocumentsInput = {
  /**
   * A unique identifier for the client performing the mutation.
   */
  clientMutationId?: string | null;
  document_categories: DocumentCategoryInput[];
};

/**
 * Possible document categories
 */
export enum DocumentCategories {
  WILL_OR_TRUST = "WILL_OR_TRUST",
  APPLICATION = "APPLICATION",
  BILL_OF_SALE = "BILL_OF_SALE",
  COPY_CERTIFICATION = "COPY_CERTIFICATION",
  COURT_ISSUED_DOCUMENT = "COURT_ISSUED_DOCUMENT",
  DEEDS = "DEEDS",
  DMV_FORM = "DMV_FORM",
  I9 = "I9",
  LEASE = "LEASE",
  LENDER_PACKAGE = "LENDER_PACKAGE",
  PS1583 = "PS1583",
  POA = "POA",
  TITLE_PACKAGE = "TITLE_PACKAGE",
  VITAL_RECORDS_REQUEST = "VITAL_RECORDS_REQUEST",
  CUSTOM = "CUSTOM",
}

export type DocumentCategoryInput = {
  /**
   * Category of the document
   */
  category: DocumentCategories;
  /**
   * String inputted if category is other or multiple
   */
  custom_category?: string | null;
  /**
   * ID of the document to be categorized
   */
  document_id: string;
};
        "#,
    );
}

#[test]
fn compile_mutation_with_variables() {
    basic_success_with_global_types_assert(
        "
mutation AddCheckmarkAnnotation($input: AddCheckmarkAnnotationInput!) {
  addCheckmarkAnnotation(input: $input) {
    annotation {
      id
    }
  }
}
    ",
        "AddCheckmarkAnnotation.ts",
        r#"
import type { AddCheckmarkAnnotationInput } from "__generated__/globalTypes";

export type AddCheckmarkAnnotation_addCheckmarkAnnotation_annotation = {
  id: string;
};

export type AddCheckmarkAnnotation_addCheckmarkAnnotation = {
  annotation: AddCheckmarkAnnotation_addCheckmarkAnnotation_annotation | null;
};

export type AddCheckmarkAnnotation = {
  addCheckmarkAnnotation: AddCheckmarkAnnotation_addCheckmarkAnnotation | null;
};

export type AddCheckmarkAnnotationVariables = {
  input: AddCheckmarkAnnotationInput;
};
        "#,
        r#"
export type AddCheckmarkAnnotationInput = {
  annotation_designation_id?: string | null;
  author_id?: string | null;
  /**
   * A unique identifier for the client performing the mutation.
   */
  clientMutationId?: string | null;
  document_bundle_id?: string | null;
  document_id: string;
  location: AnnotationLocationInput;
  meeting_id?: string | null;
  review_session_id?: string | null;
  size: SizeInput;
};

export type AnnotationLocationInput = {
  page: number;
  page_type?: PageTypes | null;
  /**
   * Top left coordinate
   */
  point?: PointInput | null;
};

/**
 * Type of page specified; unless DOCUMENT refers to specialized notary legal page
 */
export enum PageTypes {
  DOCUMENT = "DOCUMENT",
  CERTIFICATE_OF_ACKNOWLEDGEMENT = "CERTIFICATE_OF_ACKNOWLEDGEMENT",
  COPY_CERTIFICATION = "COPY_CERTIFICATION",
  JURAT = "JURAT",
  CERTIFICATE_OF_AUTHORITY = "CERTIFICATE_OF_AUTHORITY",
  STATE_AL = "STATE_AL",
  STATE_AK = "STATE_AK",
  STATE_AZ = "STATE_AZ",
  STATE_AR = "STATE_AR",
  STATE_CA = "STATE_CA",
  STATE_CO = "STATE_CO",
  STATE_CT = "STATE_CT",
  STATE_DE = "STATE_DE",
  STATE_DC = "STATE_DC",
  STATE_FL = "STATE_FL",
  STATE_GA = "STATE_GA",
  STATE_HI = "STATE_HI",
  STATE_ID = "STATE_ID",
  STATE_IL = "STATE_IL",
  STATE_IN = "STATE_IN",
  STATE_IA = "STATE_IA",
  STATE_KS = "STATE_KS",
  STATE_KY = "STATE_KY",
  STATE_LA = "STATE_LA",
  STATE_ME = "STATE_ME",
  STATE_MD = "STATE_MD",
  STATE_MA = "STATE_MA",
  STATE_MI = "STATE_MI",
  STATE_MN = "STATE_MN",
  STATE_MS = "STATE_MS",
  STATE_MO = "STATE_MO",
  STATE_MT = "STATE_MT",
  STATE_NE = "STATE_NE",
  STATE_NV = "STATE_NV",
  STATE_NH = "STATE_NH",
  STATE_NJ = "STATE_NJ",
  STATE_NM = "STATE_NM",
  STATE_NY = "STATE_NY",
  STATE_NC = "STATE_NC",
  STATE_ND = "STATE_ND",
  STATE_OH = "STATE_OH",
  STATE_OK = "STATE_OK",
  STATE_OR = "STATE_OR",
  STATE_PA = "STATE_PA",
  STATE_RI = "STATE_RI",
  STATE_SC = "STATE_SC",
  STATE_SD = "STATE_SD",
  STATE_TN = "STATE_TN",
  STATE_TX = "STATE_TX",
  STATE_UT = "STATE_UT",
  STATE_VT = "STATE_VT",
  STATE_VA = "STATE_VA",
  STATE_WA = "STATE_WA",
  STATE_WV = "STATE_WV",
  STATE_WI = "STATE_WI",
  STATE_WY = "STATE_WY",
}

export type PointInput = {
  x: number;
  y: number;
};

export type SizeInput = {
  height: number;
  width: number;
};
        "#,
    );
}
