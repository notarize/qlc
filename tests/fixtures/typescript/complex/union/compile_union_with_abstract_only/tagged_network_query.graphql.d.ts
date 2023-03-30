import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetTagsForNetwork_network_tags_author_tags = {
  readonly timeToLiveMs: number;
};

export type GetTagsForNetwork_network_tags_author = {
  readonly __typename: "User";
  readonly id: string;
  readonly tags: GetTagsForNetwork_network_tags_author_tags[];
};

export type GetTagsForNetwork_network_tags = {
  readonly __typename: "BooleanTag" | "JSONTag" | "KeyValueTag";
  readonly author: GetTagsForNetwork_network_tags_author;
  readonly timeToLiveMs: number;
};

export type GetTagsForNetwork_network = {
  readonly id: string;
  readonly tags: GetTagsForNetwork_network_tags[];
};

export type GetTagsForNetwork = {
  readonly network: GetTagsForNetwork_network | null;
};

declare const graphqlDocument: QueryDocumentNode<GetTagsForNetwork, never>;
export default graphqlDocument;
