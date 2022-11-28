export type DeeplyFragmented_operator = {
  firstName: string;
  id: string;
  id2: string;
  id3: string;
  idDeep: string;
  lastName: string;
};

export type DeeplyFragmented = {
  operator: DeeplyFragmented_operator | null;
};
