import type { QueryDocumentNode } from "@web/typed_node";
import type { OperatingSystem } from "@web/graphql_globals";

export type UsingModuleConfig_operator_personalHost = {
  readonly id: string;
  readonly numCpus: number;
  readonly operatingSystem: OperatingSystem;
};

export type UsingModuleConfig_operator = {
  readonly id: string;
  /**
   * A user's personal device
   */
  readonly personalHost: UsingModuleConfig_operator_personalHost;
};

export type UsingModuleConfig = {
  readonly operator: UsingModuleConfig_operator | null;
};

declare const graphqlDocument: QueryDocumentNode<UsingModuleConfig, never>;
export default graphqlDocument;
