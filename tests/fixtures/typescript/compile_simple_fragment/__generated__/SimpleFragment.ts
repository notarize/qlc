export type SimpleFragment_manager = {
  readonly firstName: string;
  readonly id: string;
};

export type SimpleFragment = {
  readonly email: string;
  readonly id: string;
  /**
   * A user's manager, if they have one
   */
  readonly manager: SimpleFragment_manager | null;
};
