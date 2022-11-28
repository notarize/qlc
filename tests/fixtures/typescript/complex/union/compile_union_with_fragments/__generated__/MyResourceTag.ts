export type MyResourceTag_BooleanTag_author = {
  email: string;
  id: string;
};

export type MyResourceTag_BooleanTag = {
  __typename: "BooleanTag";
  author: MyResourceTag_BooleanTag_author;
  name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  not: boolean;
  timeToLiveMs: number;
};

export type MyResourceTag_KeyValueTag_author = {
  email: string;
  id: string;
};

export type MyResourceTag_KeyValueTag = {
  __typename: "KeyValueTag";
  anotherKeyName: string;
  author: MyResourceTag_KeyValueTag_author;
  key: string;
  keyTTL: number;
  timeToLiveMs: number;
  value: string;
};

export type MyResourceTag_$$other_author = {
  email: string;
  id: string;
};

export type MyResourceTag_$$other = {
  __typename: "JSONTag";
  author: MyResourceTag_$$other_author;
  timeToLiveMs: number;
};

export type MyResourceTag = MyResourceTag_BooleanTag | MyResourceTag_KeyValueTag | MyResourceTag_$$other;
