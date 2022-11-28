export type SimpleGetNetwork_network = {
  cidr: string;
  id: string;
};

export type SimpleGetNetwork = {
  network: SimpleGetNetwork_network | null;
};

export type SimpleGetNetworkVariables = {
  networkId: string;
};
