👌 QL Compiler (qlc)
--------------------

The QL compiler is a fun codegenerator for GraphQL clients. Specifically, it is capable of
reading `.graphql` query, mutation, and fragment files and combining this with schema introspection JSON to
produce ad-hoc type definitions for TypeScript. It is similar to the tools [Apollo Tooling CLI](https://github.com/apollographql/apollo-tooling)
and [GraphQL Code Generator](https://github.com/dotansimha/graphql-code-generator), but smaller in scope
(and much faster).

### Motivating Example

Say you have a query that looks like this:

```graphql
query CommentQuery($id: ID!) {
  comment(id: $id) {
    author {
      name
      avatar: profilePictureUrl
    }
    content
  }
}
```

If you are using TypeScript and a GraphQL client, it would be useful to get the type of this query. You could
write one out by hand (and then maintain this definition as the query changes). But since GraphQL supports
introspection and has a schema, we already know the type for the above! `qlc` enables you to automate the
codegen of the following types:

```ts
export interface CommentQuery_comment_author {
  name: string;
  avatar: string | null;
}

export interface CommentQuery_comment {
  author: CommentQuery_comment_author;
  content: string;
}

export interface CommentQuery {
  comment: CommentQuery_comment;
}

export interface CommentQueryVariables {
  id: string;
}
```

### Usage

You can download the latest binaries (currently available for Linux and MacOS) on the
[releases](https://github.com/notarize/qlc/releases) page.

For convenience, it is also available as an NPM package:

```sh
$ yarn add @notarize/qlc-cli
$ yarn run qlc --help
```

`qlc` will recursively scan directories, finding `.graphql` files and produce `.ts` files in the same
modules under a `__generated__` submodule. By default, it starts at the working directory but you can
optionally provide it a directory argument. You will need to supply `qlc` with the JSON result of
_the_ introspection query. Most, if not all, GraphQL servers support producing this query result, and
the canonical implementation can even be found in the official [graphql](https://www.npmjs.com/package/graphql)
NPM package. See [this blog post](https://blog.apollographql.com/three-ways-to-represent-your-graphql-schema-a41f4175100d)
for more information as well as [this gist](https://gist.github.com/dairyisscary/5d6f0a240593560c7a0a4db08df52e36)
for an example of how to download a schema.

### Benchmarking

How much faster is "faster"? All results below are collected on MacOS, 2.8 GHz quad-core machine with 
an NVMe storage device, with the operating system's IO cache hot. The directory in question has 4241 files
and 265 `.graphql` files.

| Tool | Version | Command | Time (Wall Clock) | NPM Dependencies |
| ---- | ------- | ------- | ----------------- | ---------------- |
| qlc | 0.1.1 | `qlc src -s src/graph_artifacts/schema.json` | 0.074 sec | 1 |
| apollo | 2.12.5 (node 10.14.0) | `apollo client:codegen --target=typescript` | 1 min 33.77 sec | 330 |
