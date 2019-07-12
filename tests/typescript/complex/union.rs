use crate::helpers::basic_success_assert;

fn test_document_bundle_annotations(query_content: &str, expected_compile: &str) {
    let mut expected = String::from(expected_compile.trim());
    expected.push_str(
        "

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges {
  /**
   * The item at the end of the edge.
   */
  node: TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node | null;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations {
  /**
   * A list of edges.
   */
  edges: (TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges | null)[] | null;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node {
  /**
   * Visible annotations for a document
   */
  annotations: TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations | null;
  id: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges {
  /**
   * The item at the end of the edge.
   */
  node: TestQuery_meeting_Meeting_document_bundle_documents_edges_node | null;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents {
  /**
   * A list of edges.
   */
  edges: (TestQuery_meeting_Meeting_document_bundle_documents_edges | null)[] | null;
}

export interface TestQuery_meeting_Meeting_document_bundle {
  documents: TestQuery_meeting_Meeting_document_bundle_documents | null;
}

export interface TestQuery_meeting_Meeting {
  document_bundle: TestQuery_meeting_Meeting_document_bundle | null;
}

export type TestQuery_meeting = TestQuery_meeting_Meeting;

export interface TestQuery {
  /**
   * Fetches an object given its ID.
   */
  meeting: TestQuery_meeting | null;
}
        ");

    basic_success_assert(
        format!(
            "
query TestQuery {{
  meeting: node {{
    ... on Meeting {{
      document_bundle {{
        documents {{
          edges {{
            node {{
              id
              annotations {{
                edges {{
                  node {{
                    {}
                  }}
                }}
              }}
            }}
          }}
        }}
      }}
    }}
  }}
}}
        ",
            query_content
        )
        .as_ref(),
        "TestQuery.ts",
        Box::leak(expected.into_boxed_str()),
    );
}

#[test]
fn compile_union_with_only_implementing_types() {
    test_document_bundle_annotations(
        "
... on CheckmarkAnnotation {
  author_id
}
... on TextAnnotation {
  text
}
        ",
        r#"
export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation {
  __typename: "CheckmarkAnnotation";
  author_id: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation {
  __typename: "ImageAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation {
  __typename: "TextAnnotation";
  text: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation {
  __typename: "VectorGraphicAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation {
  __typename: "WhiteboxAnnotation";
}

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation;
       "#,
    );
}

#[test]
fn compile_union_with_typenames() {
    test_document_bundle_annotations(
        "
__typename
as: __typename
... on CheckmarkAnnotation {
  author_id
}
... on TextAnnotation {
  text
}
        ",
        r#"
export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation {
  __typename: "CheckmarkAnnotation";
  as: "CheckmarkAnnotation";
  author_id: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation {
  __typename: "ImageAnnotation";
  as: "ImageAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation {
  __typename: "TextAnnotation";
  as: "TextAnnotation";
  text: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation {
  __typename: "VectorGraphicAnnotation";
  as: "VectorGraphicAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation {
  __typename: "WhiteboxAnnotation";
  as: "WhiteboxAnnotation";
}

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation;
       "#,
    );
}

#[test]
fn compile_union_with_interface_only() {
    test_document_bundle_annotations(
        "
... on AnnotationFields {
  id
}
        ",
        r#"
export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation {
  __typename: "CheckmarkAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation {
  __typename: "ImageAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation {
  __typename: "TextAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation {
  __typename: "VectorGraphicAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation {
  __typename: "WhiteboxAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_AnnotationFields {
  id: string;
}

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = (TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation) & TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_AnnotationFields;
       "#,
    );
}

#[test]
fn compile_union_with_interface_and_implementing_types() {
    test_document_bundle_annotations(
        "
... on AnnotationFields {
  id
}

... on TextAnnotation {
  author_id
}
        ",
        r#"
export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation {
  __typename: "CheckmarkAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation {
  __typename: "ImageAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation {
  __typename: "TextAnnotation";
  author_id: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation {
  __typename: "VectorGraphicAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation {
  __typename: "WhiteboxAnnotation";
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_AnnotationFields {
  id: string;
}

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = (TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_ImageAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_VectorGraphicAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_WhiteboxAnnotation) & TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_AnnotationFields;
       "#,
    );
}

#[test]
fn compile_union_with_fragments() {
    basic_success_assert(
        "
fragment meetingAnnotationFragment on Annotation {
  __typename
  ... on AnnotationFields {
    id
  }
  ... on TextAnnotation {
    text
  }
}
        ",
        "meetingAnnotationFragment.ts",
        r#"
export interface meetingAnnotationFragment_CheckmarkAnnotation {
  __typename: "CheckmarkAnnotation";
}

export interface meetingAnnotationFragment_ImageAnnotation {
  __typename: "ImageAnnotation";
}

export interface meetingAnnotationFragment_TextAnnotation {
  __typename: "TextAnnotation";
  text: string;
}

export interface meetingAnnotationFragment_VectorGraphicAnnotation {
  __typename: "VectorGraphicAnnotation";
}

export interface meetingAnnotationFragment_WhiteboxAnnotation {
  __typename: "WhiteboxAnnotation";
}

export interface meetingAnnotationFragment_AnnotationFields {
  id: string;
}

export type meetingAnnotationFragment = (meetingAnnotationFragment_CheckmarkAnnotation | meetingAnnotationFragment_ImageAnnotation | meetingAnnotationFragment_TextAnnotation | meetingAnnotationFragment_VectorGraphicAnnotation | meetingAnnotationFragment_WhiteboxAnnotation) & meetingAnnotationFragment_AnnotationFields;
        "#,
    );
}
