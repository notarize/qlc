import type { OperatingSystem } from "~/gen-me/global-types";

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
