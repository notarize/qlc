import type { MutationDocumentNode } from "@notarize/qlc-cli/typed-documentnode";
import type { ProvisionHostInput } from "graphql-globals";

export type SimpleProvision_provisionHost_host = {
  readonly id: string;
};

export type SimpleProvision_provisionHost = {
  readonly host: SimpleProvision_provisionHost_host;
};

export type SimpleProvision = {
  /**
   * Null return means it failed
   */
  readonly provisionHost: SimpleProvision_provisionHost | null;
};

export type SimpleProvisionVariables = {
  input: ProvisionHostInput;
};

declare const graphqlDocument: MutationDocumentNode<SimpleProvision, SimpleProvisionVariables>;
export default graphqlDocument;
