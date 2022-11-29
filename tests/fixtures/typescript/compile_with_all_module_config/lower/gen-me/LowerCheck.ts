import type { OperatingSystem } from "~/gen-me/global-types";

export type LowerCheck_personalHost = {
  readonly operatingSystem: OperatingSystem;
};

export type LowerCheck = {
  readonly id: string;
  /**
   * A user's personal device
   */
  readonly personalHost: LowerCheck_personalHost;
};
