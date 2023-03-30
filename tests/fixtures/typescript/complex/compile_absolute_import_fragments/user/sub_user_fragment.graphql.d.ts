import type { FragmentDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type SubUser = {
  readonly email: string;
};

declare const graphqlDocument: FragmentDocumentNode<SubUser>;
export default graphqlDocument;
