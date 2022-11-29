export type WithDeprecatedFields_operator = {
  readonly id: string;
  /**
   * User's last time logging in
   * @deprecated
   */
  readonly lastLogin: any | null;
  /**
   * @deprecated
   */
  readonly publicRSAKey: string;
};

export type WithDeprecatedFields = {
  readonly operator: WithDeprecatedFields_operator | null;
};
