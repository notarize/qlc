export type GetTagTypeNamesForHost_host_tags_BooleanTag = {
  readonly __typename: "BooleanTag";
  readonly as: "BooleanTag";
  readonly name: string;
};

export type GetTagTypeNamesForHost_host_tags_JSONTag = {
  readonly __typename: "JSONTag";
  readonly as: "JSONTag";
  readonly content: any;
  readonly typername: "JSONTag";
};

export type GetTagTypeNamesForHost_host_tags_$$other = {
  readonly __typename: "KeyValueTag";
  readonly as: "KeyValueTag";
};

export type GetTagTypeNamesForHost_host_tags = GetTagTypeNamesForHost_host_tags_BooleanTag | GetTagTypeNamesForHost_host_tags_JSONTag | GetTagTypeNamesForHost_host_tags_$$other;

export type GetTagTypeNamesForHost_host = {
  readonly tags: GetTagTypeNamesForHost_host_tags[];
};

export type GetTagTypeNamesForHost = {
  readonly host: GetTagTypeNamesForHost_host | null;
};
