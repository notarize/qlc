import type { MutationDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { AttachHostToNetworksInput } from "graphql-globals";

export type AddHostToManyNetworks_attachHostToNetworks_host_networks = {
  readonly id: string;
};

export type AddHostToManyNetworks_attachHostToNetworks_host = {
  readonly id: string;
  readonly networks: AddHostToManyNetworks_attachHostToNetworks_host_networks[];
};

export type AddHostToManyNetworks_attachHostToNetworks = {
  readonly host: AddHostToManyNetworks_attachHostToNetworks_host;
};

export type AddHostToManyNetworks = {
  readonly attachHostToNetworks: AddHostToManyNetworks_attachHostToNetworks;
};

export type AddHostToManyNetworksVariables = {
  input: AttachHostToNetworksInput;
};

declare const graphqlDocument: MutationDocumentNode<AddHostToManyNetworks, AddHostToManyNetworksVariables>;
export default graphqlDocument;
