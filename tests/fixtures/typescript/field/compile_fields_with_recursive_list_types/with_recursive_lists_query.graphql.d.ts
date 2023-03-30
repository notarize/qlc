import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type WithRecursiveListFields_network = {
  readonly hostIdGroups: (string[])[];
  readonly hostIdTopology: (((((string | null)[] | null) | null)[] | null) | null)[] | null;
  readonly id: string;
};

export type WithRecursiveListFields = {
  readonly network: WithRecursiveListFields_network | null;
};

declare const graphqlDocument: QueryDocumentNode<WithRecursiveListFields, never>;
export default graphqlDocument;
