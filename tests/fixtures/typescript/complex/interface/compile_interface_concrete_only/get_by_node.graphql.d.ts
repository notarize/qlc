import type { QueryDocumentNode } from "@notarize/qlc-cli/typed-documentnode";

export type GetByNodeConcreteOnly_desiredUser_User = {
  readonly email: string;
  readonly firstName: string;
};

export type GetByNodeConcreteOnly_desiredUser_$$other = {

};

export type GetByNodeConcreteOnly_desiredUser = GetByNodeConcreteOnly_desiredUser_User | GetByNodeConcreteOnly_desiredUser_$$other;

export type GetByNodeConcreteOnly = {
  readonly desiredUser: GetByNodeConcreteOnly_desiredUser | null;
};

declare const graphqlDocument: QueryDocumentNode<GetByNodeConcreteOnly, never>;
export default graphqlDocument;
