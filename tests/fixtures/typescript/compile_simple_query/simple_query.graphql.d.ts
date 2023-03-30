import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type SimpleQuery_me = {
  readonly firstName: string;
  readonly id: string;
  readonly last: string;
};

export type SimpleQuery = {
  readonly me: SimpleQuery_me | null;
};

declare const graphqlDocument: QueryDocumentNode<SimpleQuery, never>;
export default graphqlDocument;
