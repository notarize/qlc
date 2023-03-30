import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetByNodeAbstractOnly_host = {
  readonly id: string;
};

export type GetByNodeAbstractOnly = {
  readonly host: GetByNodeAbstractOnly_host | null;
};

declare const graphqlDocument: QueryDocumentNode<GetByNodeAbstractOnly, never>;
export default graphqlDocument;
