import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type SimpleSubscription_me = {
  readonly firstName: string;
  readonly id: string;
  readonly last: string;
};

export type SimpleSubscription = {
  readonly me: SimpleSubscription_me | null;
};

declare const graphqlDocument: QueryDocumentNode<SimpleSubscription, never>;
export default graphqlDocument;
