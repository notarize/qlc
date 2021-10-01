👌 QL Compiler (qlc)
--------------------

The QL compiler is a fun codegenerator for GraphQL clients. Specifically, it is capable of
reading `.graphql` query, mutation, and fragment files and combining this with schema introspection JSON to
produce ad-hoc type definitions for TypeScript. It is similar to the tools [Apollo Tooling CLI](https://github.com/apollographql/apollo-tooling)
and [GraphQL Code Generator](https://github.com/dotansimha/graphql-code-generator), but smaller in scope
(and faster).

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
export type CommentQuery_comment_author = {
  name: string;
  avatar: string | null;
};

export type CommentQuery_comment = {
  author: CommentQuery_comment_author;
  content: string;
};

export type CommentQuery = {
  comment: CommentQuery_comment;
};

export type CommentQueryVariables = {
  id: string;
};
```

### Usage

You can download _some_ prebuilt binaries on the
[releases](https://github.com/notarize/qlc/releases) page. You will need to build from source with `cargo` for other platforms.

For convenience, it is also available as an NPM package that supports x64/aarch64 MacOS and x64 linux:

```sh
$ yarn add @notarize/qlc-cli
```

`qlc` will recursively scan directories, finding `.graphql` files and produce `.ts` files in the same
modules under a `__generated__` submodule. By default, it starts at the working directory but you can
optionally provide it a directory argument. `qlc` supports fragment imports with the `#import "<file>.graphql"`
syntax at the top of your files; it supports both relative imports and absolute imports starting at the
root directory supplied to `qlc`.

You will need to supply `qlc` with the JSON result of _the_ introspection query. Most, if not all,
GraphQL servers support producing this query result, and the canonical implementation can even be found
in the official [graphql](https://www.npmjs.com/package/graphql) NPM package. See [this blog
post](https://blog.apollographql.com/three-ways-to-represent-your-graphql-schema-a41f4175100d) for more
information. For simplicity, the NPM package comes with a helper script that should be suitable for most users. See below.

#### Example

```sh
# Download a schema JSON from an endpoint and write to my_schema.json
$ yarn run qlc-download-schema https://<FQDN>/graphql my_schema.json

# Run qlc searching the src/ directory with schema JSON located at my_schema.json
$ yarn run qlc -s my_schema.json src

# There are some other options available for more complex requirements.
$ yarn run qlc --help
```

Many of the options can also be configured through a camelcased JSON file (by default `.qlcrc.json`). For example:

```json
{ "useCustomScalars": true, "numThreads": 2 }
```

### Benchmarking

How much faster is "faster"? All results below are collected on MacOS, a 2.8 GHz quad-core machine with
an NVMe storage device, with the operating system's IO cache hot. The `hyperfine` utility measured runtime.
The directory in question has 4523 files and 534 `.graphql` files.

| Command | Version | Mean Time ± σ | NPM Dependencies |
| ------- | ------- | ------------- | ---------------- |
| `qlc` | 0.6.0 | 118.8 ms ± 10.8 ms | 1 (itself) |
| `apollo client:codegen --target=typescript` | 2.31.1 (node 14.15.0) | 4.817 s ± 0.475 s | 355 |
