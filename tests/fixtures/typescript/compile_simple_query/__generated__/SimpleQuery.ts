export type SimpleQuery_me = {
  readonly firstName: string;
  readonly id: string;
  readonly last: string;
};

export type SimpleQuery = {
  readonly me: SimpleQuery_me | null;
};
