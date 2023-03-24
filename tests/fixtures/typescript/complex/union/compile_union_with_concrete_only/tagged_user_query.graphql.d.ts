import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetTagsForCurrentOperator_operator_tags_BooleanTag = {
  readonly __typename: "BooleanTag";
  readonly name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  readonly not: boolean;
  /**
   * If true, this boolean tag has inverted meaning
   */
  readonly notNot: boolean;
};

export type GetTagsForCurrentOperator_operator_tags_JSONTag = {
  readonly __typename: "JSONTag";
  readonly content: any;
};

export type GetTagsForCurrentOperator_operator_tags_KeyValueTag = {
  readonly __typename: "KeyValueTag";
  readonly key: string;
  readonly value: string;
};

export type GetTagsForCurrentOperator_operator_tags = GetTagsForCurrentOperator_operator_tags_BooleanTag | GetTagsForCurrentOperator_operator_tags_JSONTag | GetTagsForCurrentOperator_operator_tags_KeyValueTag;

export type GetTagsForCurrentOperator_operator = {
  readonly email: string;
  readonly tags: GetTagsForCurrentOperator_operator_tags[];
};

export type GetTagsForCurrentOperator = {
  readonly operator: GetTagsForCurrentOperator_operator | null;
};

declare const graphqlDocument: QueryDocumentNode<GetTagsForCurrentOperator, never>;
export default graphqlDocument;
