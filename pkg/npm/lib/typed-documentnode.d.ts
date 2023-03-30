import type { DocumentNode } from "graphql";

type BasicObject = Record<string, unknown>;

/** A query GraphQL document */
export type QueryDocumentNode<
  Data extends BasicObject,
  Variables extends BasicObject
> = DocumentNode & {
  /** @private */
  readonly __typedDocumentNodeQuery?: (v: Variables) => Data;
};

/** A subscription GraphQL document */
export type SubscriptionDocumentNode<
  Data extends BasicObject,
  Variables extends BasicObject
> = DocumentNode & {
  /** @private */
  readonly __typedDocumentNodeSubscription?: (v: Variables) => Data;
};

/** A mutation GraphQL document */
export type MutationDocumentNode<
  Data extends BasicObject,
  Variables extends BasicObject
> = DocumentNode & {
  /** @private */
  readonly __typedDocumentNodeMutation?: (v: Variables) => Data;
};

/** A fragment GraphQL document */
export type FragmentDocumentNode<Data extends BasicObject> = DocumentNode & {
  /** @private */
  readonly __typedDocumentNodeFragment?: () => Data;
};

/**
 * Helper for extracting a data type from a typed DocumentNode
 * @example
 * import MyQuery from "./query.graphql";
 * // MyQuery is QueryDocumentNode<D, V>
 * type MyQuerysData = DataOf<typeof MyQuery>; // MyQuerysData is now type D
 */
export type DataOf<T> = T extends QueryDocumentNode<infer Data, infer Variables>
  ? Data
  : T extends MutationDocumentNode<infer Data, infer Variables>
  ? Data
  : T extends SubscriptionDocumentNode<infer Data, infer Variables>
  ? Data
  : T extends FragmentDocumentNode<infer Data>
  ? Data
  : never;

/**
 * Helper for extracting a data type from a typed DocumentNode
 * @example
 * import MyMutation from "./mutation.graphql";
 * // MyMutation is MutationDocumentNode<D, V>
 * type MyMutationsVars = VariablesOf<typeof MyMutation>; // MyMutationsVars is now type V
 */
export type VariablesOf<T> = T extends QueryDocumentNode<
  infer Data,
  infer Variables
>
  ? Variables
  : T extends MutationDocumentNode<infer Data, infer Variables>
  ? Variables
  : T extends SubscriptionDocumentNode<infer Data, infer Variables>
  ? Variables
  : never;
