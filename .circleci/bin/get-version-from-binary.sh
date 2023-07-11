#!/usr/bin/env bash
set -eo pipefail

QLC_VERSION="$("$CIRCLE_WORKING_DIRECTORY/target/x86_64-unknown-linux-musl/release/qlc" --version | sed 's/QL Compiler //g')"
export QLC_VERSION
