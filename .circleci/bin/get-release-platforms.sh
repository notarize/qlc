#!/usr/bin/env bash
set -eo pipefail

RELEASE_PLATFORMS=(
 "x86_64-unknown-linux-musl"
 "x86_64-apple-darwin"
 "aarch64-apple-darwin"
)
export RELEASE_PLATFORMS
