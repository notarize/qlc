use crate::helpers::basic_success_assert;

fn test_document_bundle_annotations(query_content: &str, expected_compile: &str) {
    let mut expected = String::from(expected_compile.trim());
    expected.push_str(
        "

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges = {
  /**
   * The item at the end of the edge.
   */
  node: TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node | null;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations = {
  /**
   * A list of edges.
   */
  edges: (TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges | null)[] | null;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node = {
  /**
   * Visible annotations for a document
   */
  annotations: TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations | null;
  id: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges = {
  /**
   * The item at the end of the edge.
   */
  node: TestQuery_meeting_Meeting_document_bundle_documents_edges_node | null;
};

export type TestQuery_meeting_Meeting_document_bundle_documents = {
  /**
   * A list of edges.
   */
  edges: (TestQuery_meeting_Meeting_document_bundle_documents_edges | null)[] | null;
};

export type TestQuery_meeting_Meeting_document_bundle = {
  documents: TestQuery_meeting_Meeting_document_bundle_documents | null;
};

export type TestQuery_meeting_Meeting = {
  document_bundle: TestQuery_meeting_Meeting_document_bundle | null;
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
                    {query_content}
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
        "
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
export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation = {
  author_id: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation = {
  text: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other = {

};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other;
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
  typername: __typename
  author_id
}
... on TextAnnotation {
  text
}
        ",
        r#"
export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation = {
  __typename: "CheckmarkAnnotation";
  as: "CheckmarkAnnotation";
  author_id: string;
  typername: "CheckmarkAnnotation";
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation = {
  __typename: "TextAnnotation";
  as: "TextAnnotation";
  text: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other = {
  __typename: "ImageAnnotation" | "VectorGraphicAnnotation" | "WhiteboxAnnotation";
  as: "ImageAnnotation" | "VectorGraphicAnnotation" | "WhiteboxAnnotation";
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other;
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
export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = {
  id: string;
};
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
export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation = {
  author_id: string;
  id: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other = {
  id: string;
};

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_$$other;
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
export type meetingAnnotationFragment_TextAnnotation = {
  __typename: "TextAnnotation";
  id: string;
  text: string;
};

export type meetingAnnotationFragment_$$other = {
  __typename: "CheckmarkAnnotation" | "ImageAnnotation" | "VectorGraphicAnnotation" | "WhiteboxAnnotation";
  id: string;
};

export type meetingAnnotationFragment = meetingAnnotationFragment_TextAnnotation | meetingAnnotationFragment_$$other;
        "#,
    );
}
