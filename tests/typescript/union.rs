use crate::helpers::basic_success_assert;

#[test]
fn compile_union() {
    basic_success_assert(
        "
query TestQuery {
  meeting: node {
    ... on Meeting {
      document_bundle {
        documents {
          edges {
            node {
              id
              annotations {
                edges {
                  node {
                    ... on CheckmarkAnnotation {
                      author_id
                    }
                    ... on TextAnnotation {
                      text
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
        ",
        "TestQuery.ts",
        "
export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation {
  author_id: string;
}

export interface TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation {
  text: string;
}

export type TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node = TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_CheckmarkAnnotation | TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations_edges_node_TextAnnotation;

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
  id: string;
  /**
   * Visible annotations for a document
   */
  annotations: TestQuery_meeting_Meeting_document_bundle_documents_edges_node_annotations | null;
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
        ",
    );
}
