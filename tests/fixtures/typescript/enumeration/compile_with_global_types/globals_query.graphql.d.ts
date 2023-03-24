import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { OperatingSystem } from "graphql-globals";

export type WithGlobals_operator_personalHost = {
  readonly id: string;
  readonly operatingSystem: OperatingSystem;
};

export type WithGlobals_operator = {
  /**
   * A user's personal device
   */
  readonly personalHost: WithGlobals_operator_personalHost;
};

export type WithGlobals = {
  readonly operator: WithGlobals_operator | null;
};

declare const graphqlDocument: QueryDocumentNode<WithGlobals, never>;
export default graphqlDocument;
