export type GetTagsForNetwork_network_tags_author_tags = {
  timeToLiveMs: number;
};

export type GetTagsForNetwork_network_tags_author = {
  __typename: "User";
  id: string;
  tags: GetTagsForNetwork_network_tags_author_tags[];
};

export type GetTagsForNetwork_network_tags = {
  __typename: "BooleanTag" | "JSONTag" | "KeyValueTag";
  author: GetTagsForNetwork_network_tags_author;
  timeToLiveMs: number;
};

export type GetTagsForNetwork_network = {
  id: string;
  tags: GetTagsForNetwork_network_tags[];
};

export type GetTagsForNetwork = {
  network: GetTagsForNetwork_network | null;
};
