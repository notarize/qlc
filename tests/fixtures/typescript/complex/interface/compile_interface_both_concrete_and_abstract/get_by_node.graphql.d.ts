import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_Network = {
  readonly __typename: "Network";
  readonly cidr: string;
  readonly id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_User = {
  readonly __typename: "User";
  readonly email: string;
  readonly firstName: string;
  readonly id: string;
  readonly lastName: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_$$other = {
  readonly __typename: "Host";
  readonly id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_justHiredOperator = GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_Network | GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_User | GetSeveralNodesWithConcreteAndAbstract_justHiredOperator_$$other;

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork_Network = {
  readonly __typename: "Network";
  readonly cidr: string;
  readonly id: string;
  readonly ipv6Cidr: string | null;
};

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork_$$other = {
  readonly __typename: "Host" | "User";
  readonly id: string;
};

export type GetSeveralNodesWithConcreteAndAbstract_someNetwork = GetSeveralNodesWithConcreteAndAbstract_someNetwork_Network | GetSeveralNodesWithConcreteAndAbstract_someNetwork_$$other;

export type GetSeveralNodesWithConcreteAndAbstract = {
  readonly justHiredOperator: GetSeveralNodesWithConcreteAndAbstract_justHiredOperator | null;
  readonly someNetwork: GetSeveralNodesWithConcreteAndAbstract_someNetwork | null;
};

declare const graphqlDocument: QueryDocumentNode<GetSeveralNodesWithConcreteAndAbstract, never>;
export default graphqlDocument;
