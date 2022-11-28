export type GetByNodeWithInterface_network123_Network_hosts = {
  totalCount: number;
};

export type GetByNodeWithInterface_network123_Network = {
  __typename: "Network";
  another: "Network";
  as: "Network";
  cidr: string;
  hosts: GetByNodeWithInterface_network123_Network_hosts;
};

export type GetByNodeWithInterface_network123_$$other = {
  __typename: "Host" | "User";
  as: "Host" | "User";
};

export type GetByNodeWithInterface_network123 = GetByNodeWithInterface_network123_Network | GetByNodeWithInterface_network123_$$other;

export type GetByNodeWithInterface = {
  network123: GetByNodeWithInterface_network123 | null;
};
