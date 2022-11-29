export type CustomScalar_operator_activity = {
  /**
   * User's last time successfully authenticating
   */
  readonly login: ISO8601 | null;
};

export type CustomScalar_operator = {
  /**
   * User's activity timestamps
   */
  readonly activity: CustomScalar_operator_activity;
  readonly id: string;
};

export type CustomScalar = {
  readonly operator: CustomScalar_operator | null;
};
