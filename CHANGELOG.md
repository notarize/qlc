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
