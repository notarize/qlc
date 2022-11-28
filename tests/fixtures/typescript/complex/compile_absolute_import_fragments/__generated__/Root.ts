import type { OperatingSystem } from "__generated__/globalTypes";

export type Root_operator_personalHost = {
  id: string;
  osFromAbsolute: OperatingSystem;
  personalHostIdFromRelative: string;
};

export type Root_operator = {
  email: string;
  id: string;
  lastNameFromRelative: string;
  /**
   * A user's personal device
   */
  personalHost: Root_operator_personalHost;
};

export type Root = {
  operator: Root_operator | null;
};
