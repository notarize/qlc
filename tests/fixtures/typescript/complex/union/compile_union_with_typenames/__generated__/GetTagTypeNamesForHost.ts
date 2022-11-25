export type GetTagTypeNamesForHost_host_tags_BooleanTag = {
  __typename: "BooleanTag";
  as: "BooleanTag";
  name: string;
};

export type GetTagTypeNamesForHost_host_tags_JSONTag = {
  __typename: "JSONTag";
  as: "JSONTag";
  content: any;
  typername: "JSONTag";
};

export type GetTagTypeNamesForHost_host_tags_$$other = {
  __typename: "KeyValueTag";
  as: "KeyValueTag";
};

export type GetTagTypeNamesForHost_host_tags = GetTagTypeNamesForHost_host_tags_BooleanTag | GetTagTypeNamesForHost_host_tags_JSONTag | GetTagTypeNamesForHost_host_tags_$$other;

export type GetTagTypeNamesForHost_host = {
  tags: GetTagTypeNamesForHost_host_tags[];
};

export type GetTagTypeNamesForHost = {
  host: GetTagTypeNamesForHost_host | null;
};
