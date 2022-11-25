export type GetTagsForCurrentOperator_operator_tags_BooleanTag = {
  __typename: "BooleanTag";
  name: string;
  /**
   * If true, this boolean tag has inverted meaning
   */
  not: boolean;
  /**
   * If true, this boolean tag has inverted meaning
   */
  notNot: boolean;
};

export type GetTagsForCurrentOperator_operator_tags_JSONTag = {
  __typename: "JSONTag";
  content: any;
};

export type GetTagsForCurrentOperator_operator_tags_KeyValueTag = {
  __typename: "KeyValueTag";
  key: string;
  value: string;
};

export type GetTagsForCurrentOperator_operator_tags = GetTagsForCurrentOperator_operator_tags_BooleanTag | GetTagsForCurrentOperator_operator_tags_JSONTag | GetTagsForCurrentOperator_operator_tags_KeyValueTag;

export type GetTagsForCurrentOperator_operator = {
  email: string;
  tags: GetTagsForCurrentOperator_operator_tags[];
};

export type GetTagsForCurrentOperator = {
  operator: GetTagsForCurrentOperator_operator | null;
};
