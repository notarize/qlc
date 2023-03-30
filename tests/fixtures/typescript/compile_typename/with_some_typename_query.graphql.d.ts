import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type WithSomeDunderDunderTypename_operator_personalHost = {
  readonly __typename: "Host";
  readonly id: string;
};

export type WithSomeDunderDunderTypename_operator = {
  readonly as: "User";
  /**
   * A user's personal device
   */
  readonly personalHost: WithSomeDunderDunderTypename_operator_personalHost;
};

export type WithSomeDunderDunderTypename = {
  readonly operator: WithSomeDunderDunderTypename_operator | null;
};

declare const graphqlDocument: QueryDocumentNode<WithSomeDunderDunderTypename, never>;
export default graphqlDocument;
