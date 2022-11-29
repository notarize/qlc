export type NoReadOnlyPlease_operator_personalHost = {
  numCpus: number;
};

export type NoReadOnlyPlease_operator = {
  id: string;
  /**
   * A user's personal device
   */
  personalHost: NoReadOnlyPlease_operator_personalHost;
};

export type NoReadOnlyPlease = {
  operator: NoReadOnlyPlease_operator | null;
};
