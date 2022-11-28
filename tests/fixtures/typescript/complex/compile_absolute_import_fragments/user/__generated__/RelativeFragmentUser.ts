import type { OperatingSystem } from "__generated__/globalTypes";

export type RelativeFragmentUser_personalHost = {
  osFromAbsolute: OperatingSystem;
  personalHostIdFromRelative: string;
};

export type RelativeFragmentUser = {
  email: string;
  lastNameFromRelative: string;
  /**
   * A user's personal device
   */
  personalHost: RelativeFragmentUser_personalHost;
};
