import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type SimpleGetNetwork_network = {
  readonly cidr: string;
  readonly id: string;
};

export type SimpleGetNetwork = {
  readonly network: SimpleGetNetwork_network | null;
};

export type SimpleGetNetworkVariables = {
  networkId: string;
};

declare const graphqlDocument: QueryDocumentNode<SimpleGetNetwork, SimpleGetNetworkVariables>;
export default graphqlDocument;
