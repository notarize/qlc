export type WithSomeDunderDunderTypename_operator_personalHost = {
  __typename: "Host";
  id: string;
};

export type WithSomeDunderDunderTypename_operator = {
  as: "User";
  /**
   * A user's personal device
   */
  personalHost: WithSomeDunderDunderTypename_operator_personalHost;
};

export type WithSomeDunderDunderTypename = {
  operator: WithSomeDunderDunderTypename_operator | null;
};
