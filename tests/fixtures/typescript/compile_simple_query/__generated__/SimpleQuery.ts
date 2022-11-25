export type SimpleQuery_me = {
  firstName: string;
  id: string;
  last: string;
};

export type SimpleQuery = {
  me: SimpleQuery_me | null;
};
