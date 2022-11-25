import type { AttachHostToNetworksInput } from "__generated__/globalTypes";

export type AddHostToManyNetworks_attachHostToNetworks_host_networks = {
  id: string;
};

export type AddHostToManyNetworks_attachHostToNetworks_host = {
  id: string;
  networks: AddHostToManyNetworks_attachHostToNetworks_host_networks[];
};

export type AddHostToManyNetworks_attachHostToNetworks = {
  host: AddHostToManyNetworks_attachHostToNetworks_host;
};

export type AddHostToManyNetworks = {
  attachHostToNetworks: AddHostToManyNetworks_attachHostToNetworks;
};

export type AddHostToManyNetworksVariables = {
  input: AttachHostToNetworksInput;
};
