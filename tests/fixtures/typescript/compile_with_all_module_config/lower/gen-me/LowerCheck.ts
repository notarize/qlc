import type { OperatingSystem } from "~/gen-me/global-types";

export type LowerCheck_personalHost = {
  operatingSystem: OperatingSystem;
};

export type LowerCheck = {
  id: string;
  /**
   * A user's personal device
   */
  personalHost: LowerCheck_personalHost;
};
