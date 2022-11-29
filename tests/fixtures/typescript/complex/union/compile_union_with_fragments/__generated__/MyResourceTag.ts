export type MyResourceTag_BooleanTag_author = {
  readonly email: string;
  readonly id: string;
};

export type MyResourceTag_BooleanTag = {
  readonly __typename: "BooleanTag";
  readonly author: MyResourceTag_BooleanTag_author;
  readonly name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  readonly not: boolean;
  readonly timeToLiveMs: number;
};

export type MyResourceTag_KeyValueTag_author = {
  readonly email: string;
  readonly id: string;
};

export type MyResourceTag_KeyValueTag = {
  readonly __typename: "KeyValueTag";
  readonly anotherKeyName: string;
  readonly author: MyResourceTag_KeyValueTag_author;
  readonly key: string;
  readonly keyTTL: number;
  readonly timeToLiveMs: number;
  readonly value: string;
};

export type MyResourceTag_$$other_author = {
  readonly email: string;
  readonly id: string;
};

export type MyResourceTag_$$other = {
  readonly __typename: "JSONTag";
  readonly author: MyResourceTag_$$other_author;
  readonly timeToLiveMs: number;
};

export type MyResourceTag = MyResourceTag_BooleanTag | MyResourceTag_KeyValueTag | MyResourceTag_$$other;
