import type { MutationDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type SimpleMutation = {
  readonly decommissionHost: boolean;
};

declare const graphqlDocument: MutationDocumentNode<SimpleMutation, never>;
export default graphqlDocument;
