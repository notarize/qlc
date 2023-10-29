# Changelog

## [4.2.0](https://github.com/notarize/qlc/compare/4.1.0...4.2.0)

### Bugfixes

- Cleanly exit from NPM post-install script

### Chores

- Upgrade to Rust 1.73.0
- Upgrade dependencies (including security fix)

## [4.1.0](https://github.com/notarize/qlc/compare/4.0.0...4.1.0)

### Features

- The NPM package now downloads tarballs from GitHub with sha256 checksums
- Added a `aarch64-unknown-linux-musl` binary release

### Chores

- Upgrade to Rust 1.71.0
- Upgrade dependencies

## [4.0.0](https://github.com/notarize/qlc/compare/3.1.0...4.0.0)

### Features

- **Breaking**: Switch to a `.graphql.d.ts` output scheme:
  - All imports now come from `.graphql` document modules (via corresponding `.graphql.d.ts` type
    def files)
  - `--generated-module-name` configuration has been removed since `__generated__` modules are no
    longer produced

```ts
// Before
import type { MyQuery } from "./__generated__/MyQuery";
// After
import type { MyQuery } from "./my-query.graphql";
```

- **Breaking**: The default for `--global-types-module-name` has changed from `globalTypes` to
  `graphql-globals`
- **Potentially Breaking**: The default export from these modules will be a "typed documentnode"
  with a default implementation provided in `@notarize/qlc-cli/typed-documentnode` -- one could
  override the `--typed-graphql-documentnode-module-name` with a backwards compatible type instead
  if this is breaking for client interfaces

### Perf

- Cut a few IO read system calls a result of dropping `__generated__` directories.

### Chores

- Upgrade to Rust 1.68.2
- Upgrade dependencies

## [3.1.0](https://github.com/notarize/qlc/compare/3.0.0...3.1.0)

### Perf

- Cut a few IO read system calls

### Chores

- Upgrade to Rust 1.66.0
- Upgrade dependencies

## [3.0.0](https://github.com/notarize/qlc/compare/2.2.0...3.0.0)

### Features

- **Breaking**: Add support for marking types as `readonly`. Can be disabled with
  `--disable-readonly-types`
- **Breaking**: Remove `tslint:disable` from output -- one can use tslintignore if still using this
  linter
- **Breaking**: Add much better support for recursively higher-order types (lists of lists, etc)
- Sort enum variants in output

### Chores

- **Breaking**: Update to clap v4, which parses qlc's cli args and produces help messaging
- Upgrade other minor deps
- Revamp integration testing, inventing QLC "test schema" and using file fixtures

## [2.2.0](https://github.com/notarize/qlc/compare/2.1.0...2.2.0)

### Features

- Fields deprecated in the schema are marked with `@deprecated` JSDoc
- "Variables" types now have sorted property names

### Bugfixes

- Fix an issue with root imports when `--root-dir-import-prefix` is empty string

## [2.1.0](https://github.com/notarize/qlc/compare/2.0.0...2.1.0)

### Features

- Add support for module name and paths in CLI and JSON config
  - `--root-dir-import-prefix` to configure a prefix on import module for build system resolve
    aliases
  - `--global-types-module-name` to configure `globalTypes` name
  - `--generated-module-name` to configure `__generated__` name

### Chores

- Upgrade to Rust 1.65.0
- Upgrade dependencies

## [2.0.0](https://github.com/notarize/qlc/compare/1.0.2...2.0.0)

### Features

- **Breaking** References made to `globalTypes` are imported via type only imports. This is a
  breaking change because a new minimum of TypeScript 3.8 is required.

### Chores

- Upgrade dependencies

## [1.0.2](https://github.com/notarize/qlc/compare/1.0.1...1.0.2)

### Chores

- Switched to `std::thread::scope` for threading, removing need for `Arc`
- Upgrade dependencies and Rust to 1.63.0
- Major upgrade of `assert_cmd` from v1 -> v2

## [1.0.1](https://github.com/notarize/qlc/compare/1.0.0...1.0.1)

### Chores

- Upgrade dependencies and Rust to 1.62.0

## [1.0.0](https://github.com/notarize/qlc/compare/0.10.0...1.0.0)

### Chores

- Upgrade dependencies and Rust to 1.58.1
- Major upgrade of `clap` from v2 -> v3

## [0.10.0](https://github.com/notarize/qlc/compare/0.9.0...0.10.0)

### Features

- Add prebuilt binaries and npm package support for aarch64 darwin.

## [0.9.0](https://github.com/notarize/qlc/compare/0.8.0...0.9.0)

### Features

- Add support for a `.qlcrc.json` config file, allowing most CLI args to be passed in camel-case
  JSON form (CLI args always have precedence) #19
- Add option `--show-deprecation-warnings` to have QLC print warning about usage of fields that are
  deprecated in the schema #22

### Chores

- Upgrade dependencies and Rust to 1.55
- Remove usage of `Mutex`/`Arc` for worker aggregates for less contention (performance in some
  cases) and a large drop in the number of `.unwrap()` calls
- Upgrade github release client used during CI (0.12.2 -> 0.14.0)

## [0.8.0](https://github.com/notarize/qlc/compare/0.7.0...0.8.0)

### Bugfixes

- Fix compiler error message for unknown variable types.

### Chores

- Upgrade dependencies and Rust to 1.51

## [0.7.0](https://github.com/notarize/qlc/compare/0.6.1...0.7.0)

### Features

- Always use deterministic output to prevent diffs in file signatures
  - This is better compatiblity with tsserver for instance
- Add support for subscriptions

### Bugfixes

- Fix some typos in error messages and make some help text more clear
- Reduce the aggressiveness of the similarity check for help

## [0.6.1](https://github.com/notarize/qlc/compare/0.6.0...0.6.1)

### Bugfixes

- Add missing binary NPM link in package

## [0.6.0](https://github.com/notarize/qlc/compare/0.5.0...0.6.0)

### Features

- Improve error CLI messages
- Add support for warning CLI messages
- Add download schema script to NPM package

### Bugfixes

- Reduce `unwrap`/`expect`/`panic!` calls for more robustness in failure modes

### Chores

- Upgrade dependencies and Rust to 1.47

  - Includes major upgrade of `graphql-parser` and `crossbeam-channel`

## [0.5.0](https://github.com/notarize/qlc/compare/0.4.0...0.5.0)

### Bugfixes

- Support custom scalars in the variable position

### Chores

- Upgrade dependencies and Rust to 1.44

## [0.4.0](https://github.com/notarize/qlc/compare/0.3.0...0.4.0)

### Features

- New IR for better potential for future target langs
- Switch from `interface` to `type` for _all_ declarations

### Bugfixes

- Utilize new IR for fixes for union and interface types

### Chores

- Upgrade dependencies and Rust to 1.42

## [0.3.0](https://github.com/notarize/qlc/compare/0.2.0...0.3.0)

### Features

- Add `--custom-scalar-prefix` argument

## [0.2.0](https://github.com/notarize/qlc/compare/0.1.2...0.2.0)

### Features

- Add `--use-custom-scalars` argument

### Bugfixes

- Add support for Node v8

### Chores

- Upgrade to Rust 1.38.0

## [0.1.2](https://github.com/notarize/qlc/compare/0.1.1...0.1.2)

### Features

- Add some CLI help information

## [0.1.1](https://github.com/notarize/qlc/compare/0.1.0...0.1.1)

### Bugfixes

- Fix for NPM package binary link

## [0.1.0](https://github.com/notarize/qlc/tree/0.1.0)

Initial public release!
