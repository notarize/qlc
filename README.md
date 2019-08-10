# QL Compiler

The QL compiler `qlc` is a super fast and fun codegenerator for GraphQL clients. Specifically, it is capable of
reading `.graphql` queries, mutations, and fragment files and combining this with introspection schema JSON to
produce ad-hoc type defintions for TypeScript. Its similar to tools the [Apollo Tooling CLI](https://github.com/apollographql/apollo-tooling)
and [GraphQL Code Generator](https://github.com/dotansimha/graphql-code-generator), but smaller in scope
(and much faster).

## Example

Say you have a query that looks like this:

```graphql
query MyQuery {
  comment {
    author {
      name
    }
    content
  }
}
```

Using qlc would enable you to codegen the following TypeScript file easily:

```ts
export interface MyQuery_comment_author {
  name: string | null;
}

export interface MyQuery_comment {
  author: MyQuery_comment_author;
  content: string;
}

export interface MyQuery {
  comment: MyQuery_comment;
}
```

## Usage

You can use the binary as is see `--help` for various CLI options (there are not many). Additionally,
since you are likely using this from a node project, there is a convience NPM package available.

```sh
yarn add @notarize/qlc-cli
```
