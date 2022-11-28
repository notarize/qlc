export type SimpleFragment_manager = {
  firstName: string;
  id: string;
};

export type SimpleFragment = {
  email: string;
  id: string;
  /**
   * A user's manager, if they have one
   */
  manager: SimpleFragment_manager | null;
};
