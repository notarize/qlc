import type { ProvisionHostInput } from "__generated__/globalTypes";

export type SimpleProvision_provisionHost_host = {
  id: string;
};

export type SimpleProvision_provisionHost = {
  host: SimpleProvision_provisionHost_host;
};

export type SimpleProvision = {
  /**
   * Null return means it failed
   */
  provisionHost: SimpleProvision_provisionHost | null;
};

export type SimpleProvisionVariables = {
  input: ProvisionHostInput;
};
