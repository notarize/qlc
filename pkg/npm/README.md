# @notarize/qlc-cli

This is an npm module for using [qlc](https://github.com/notarize/qlc) (a rust binary) in a Node project.

### How it works

- QLC is built in CI and binaries are posted with a GitHub release.
- In this module's postinstall task, it determines which platform it is being installed on and downloads the correct binary.
