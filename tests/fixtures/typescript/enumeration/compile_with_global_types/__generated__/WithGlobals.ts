import type { OperatingSystem } from "__generated__/globalTypes";

export type WithGlobals_operator_personalHost = {
  readonly id: string;
  readonly operatingSystem: OperatingSystem;
};

export type WithGlobals_operator = {
  /**
   * A user's personal device
   */
  readonly personalHost: WithGlobals_operator_personalHost;
};

export type WithGlobals = {
  readonly operator: WithGlobals_operator | null;
};
