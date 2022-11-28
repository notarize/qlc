export type CustomScalar_operator_activity = {
  /**
   * User's last time successfully authenticating
   */
  login: any | null;
};

export type CustomScalar_operator = {
  /**
   * User's activity timestamps
   */
  activity: CustomScalar_operator_activity;
  id: string;
};

export type CustomScalar = {
  operator: CustomScalar_operator | null;
};
