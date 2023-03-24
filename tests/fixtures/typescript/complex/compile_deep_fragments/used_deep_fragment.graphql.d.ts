import type { FragmentDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type UserDeep = {
  readonly idDeep: string;
  readonly lastName: string;
};

declare const graphqlDocument: FragmentDocumentNode<UserDeep>;
export default graphqlDocument;
