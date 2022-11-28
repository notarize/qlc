export type WithDeprecatedFields_operator = {
  id: string;
  /**
   * User's last time logging in
   * @deprecated
   */
  lastLogin: any | null;
  /**
   * @deprecated
   */
  publicRSAKey: string;
};

export type WithDeprecatedFields = {
  operator: WithDeprecatedFields_operator | null;
};
