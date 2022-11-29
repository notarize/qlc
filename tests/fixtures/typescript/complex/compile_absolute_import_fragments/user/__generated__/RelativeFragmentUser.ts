import type { OperatingSystem } from "__generated__/globalTypes";

export type RelativeFragmentUser_personalHost = {
  readonly osFromAbsolute: OperatingSystem;
  readonly personalHostIdFromRelative: string;
};

export type RelativeFragmentUser = {
  readonly email: string;
  readonly lastNameFromRelative: string;
  /**
   * A user's personal device
   */
  readonly personalHost: RelativeFragmentUser_personalHost;
};
