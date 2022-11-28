export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_Network = {
  __typename: "Network";
  cidr: string;
  id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_User = {
  __typename: "User";
  email: string;
  firstName: string;
  id: string;
  lastName: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_$$other = {
  __typename: "Host";
  id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator = GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_Network | GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_User | GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_$$other;

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork_Network = {
  __typename: "Network";
  cidr: string;
  id: string;
  ipv6Cidr: string | null;
};

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork_$$other = {
  __typename: "Host" | "User";
  id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork = GetSeveralNodesWithConcreteAndAbstract_someNetwork_Network | GetSeveralNodesWithConcreteAndAbstract_someNetwork_$$other;

export type GetSeveralNodesWithConcreteAndAbstract = {
  justHiredOperator: GetSeveralNodesWithConcreteAndAbstract_justHiredOperator | null;
  someNetwork: GetSeveralNodesWithConcreteAndAbstract_someNetwork | null;
};
