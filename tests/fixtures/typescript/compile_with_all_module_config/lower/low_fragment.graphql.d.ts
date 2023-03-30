import type { FragmentDocumentNode } from "@web/typed_node";
import type { OperatingSystem } from "@web/graphql_globals";

export type LowerCheck_personalHost = {
  readonly operatingSystem: OperatingSystem;
};

export type LowerCheck = {
  readonly id: string;
  /**
   * A user's personal device
   */
  readonly personalHost: LowerCheck_personalHost;
};

declare const graphqlDocument: FragmentDocumentNode<LowerCheck>;
export default graphqlDocument;
