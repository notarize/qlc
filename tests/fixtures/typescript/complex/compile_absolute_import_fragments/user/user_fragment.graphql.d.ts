import type { FragmentDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { OperatingSystem } from "graphql-globals";

export type RelativeFragmentUser_personalHost = {
  readonly osFromAbsolute: OperatingSystem;
  readonly personalHostIdFromRelative: string;
};

export type RelativeFragmentUser = {
  readonly email: string;
  readonly lastNameFromRelative: string;
  /**
   * A user's personal device
   */
  readonly personalHost: RelativeFragmentUser_personalHost;
};

declare const graphqlDocument: FragmentDocumentNode<RelativeFragmentUser>;
export default graphqlDocument;
