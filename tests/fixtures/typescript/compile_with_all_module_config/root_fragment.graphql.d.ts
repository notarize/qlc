import type { FragmentDocumentNode } from "@web/typed_node";

export type RootCheck = {
  readonly id: string;
  readonly numCpus: number;
};

declare const graphqlDocument: FragmentDocumentNode<RootCheck>;
export default graphqlDocument;
