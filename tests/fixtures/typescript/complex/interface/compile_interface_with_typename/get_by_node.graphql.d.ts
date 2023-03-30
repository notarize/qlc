import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetByNodeWithInterface_network123_Network_hosts = {
  readonly totalCount: number;
};

export type GetByNodeWithInterface_network123_Network = {
  readonly __typename: "Network";
  readonly another: "Network";
  readonly as: "Network";
  readonly cidr: string;
  readonly hosts: GetByNodeWithInterface_network123_Network_hosts;
};

export type GetByNodeWithInterface_network123_$$other = {
  readonly __typename: "Host" | "User";
  readonly as: "Host" | "User";
};

export type GetByNodeWithInterface_network123 = GetByNodeWithInterface_network123_Network | GetByNodeWithInterface_network123_$$other;

export type GetByNodeWithInterface = {
  readonly network123: GetByNodeWithInterface_network123 | null;
};

declare const graphqlDocument: QueryDocumentNode<GetByNodeWithInterface, never>;
export default graphqlDocument;
