import type { FragmentDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { OperatingSystem } from "graphql-globals";

export type AbsoluteFragmentHost = {
  readonly osFromAbsolute: OperatingSystem;
};

declare const graphqlDocument: FragmentDocumentNode<AbsoluteFragmentHost>;
export default graphqlDocument;
