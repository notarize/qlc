export type GetTagsForHost_host_tags_BooleanTag = {
  readonly __typename: "BooleanTag";
  readonly name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  readonly not: boolean;
  readonly timeToLiveMs: number;
};

export type GetTagsForHost_host_tags_$$other = {
  readonly __typename: "JSONTag" | "KeyValueTag";
  readonly timeToLiveMs: number;
};

export type GetTagsForHost_host_tags = GetTagsForHost_host_tags_BooleanTag | GetTagsForHost_host_tags_$$other;

export type GetTagsForHost_host = {
  readonly id: string;
  readonly tags: GetTagsForHost_host_tags[];
};

export type GetTagsForHost = {
  readonly host: GetTagsForHost_host | null;
};
