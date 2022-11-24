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
