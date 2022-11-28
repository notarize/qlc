export type GetTagsForHost_host_tags_BooleanTag = {
  __typename: "BooleanTag";
  name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  not: boolean;
  timeToLiveMs: number;
};

export type GetTagsForHost_host_tags_$$other = {
  __typename: "JSONTag" | "KeyValueTag";
  timeToLiveMs: number;
};

export type GetTagsForHost_host_tags = GetTagsForHost_host_tags_BooleanTag | GetTagsForHost_host_tags_$$other;

export type GetTagsForHost_host = {
  id: string;
  tags: GetTagsForHost_host_tags[];
};

export type GetTagsForHost = {
  host: GetTagsForHost_host | null;
};
