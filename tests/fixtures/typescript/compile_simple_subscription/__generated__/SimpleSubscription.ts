export type SimpleSubscription_me = {
  firstName: string;
  id: string;
  last: string;
};

export type SimpleSubscription = {
  me: SimpleSubscription_me | null;
};
