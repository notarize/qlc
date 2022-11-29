export type DeeplyFragmented_operator = {
  readonly firstName: string;
  readonly id: string;
  readonly id2: string;
  readonly id3: string;
  readonly idDeep: string;
  readonly lastName: string;
};

export type DeeplyFragmented = {
  readonly operator: DeeplyFragmented_operator | null;
};
