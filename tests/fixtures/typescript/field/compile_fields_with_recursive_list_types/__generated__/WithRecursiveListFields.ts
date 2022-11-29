export type WithRecursiveListFields_network = {
  readonly hostIdGroups: (string[])[];
  readonly hostIdTopology: (((((string | null)[] | null) | null)[] | null) | null)[] | null;
  readonly id: string;
};

export type WithRecursiveListFields = {
  readonly network: WithRecursiveListFields_network | null;
};
