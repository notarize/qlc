import type { OperatingSystem } from "__generated__/globalTypes";

export type WithGlobals_operator_personalHost = {
  id: string;
  operatingSystem: OperatingSystem;
};

export type WithGlobals_operator = {
  /**
   * A user's personal device
   */
  personalHost: WithGlobals_operator_personalHost;
};

export type WithGlobals = {
  operator: WithGlobals_operator | null;
};
