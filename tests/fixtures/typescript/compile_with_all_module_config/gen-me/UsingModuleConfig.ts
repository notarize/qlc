import type { OperatingSystem } from "~/gen-me/global-types";

export type UsingModuleConfig_operator_personalHost = {
  id: string;
  numCpus: number;
  operatingSystem: OperatingSystem;
};

export type UsingModuleConfig_operator = {
  id: string;
  /**
   * A user's personal device
   */
  personalHost: UsingModuleConfig_operator_personalHost;
};

export type UsingModuleConfig = {
  operator: UsingModuleConfig_operator | null;
};
