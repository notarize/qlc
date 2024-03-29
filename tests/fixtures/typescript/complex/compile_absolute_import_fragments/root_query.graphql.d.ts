import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { OperatingSystem } from "graphql-globals";

export type Root_operator_personalHost = {
  readonly id: string;
  readonly osFromAbsolute: OperatingSystem;
  readonly personalHostIdFromRelative: string;
};

export type Root_operator = {
  readonly email: string;
  readonly id: string;
  readonly lastNameFromRelative: string;
  /**
   * A user's personal device
   */
  readonly personalHost: Root_operator_personalHost;
};

export type Root = {
  readonly operator: Root_operator | null;
};

declare const graphqlDocument: QueryDocumentNode<Root, never>;
export default graphqlDocument;
