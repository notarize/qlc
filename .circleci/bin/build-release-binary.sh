#!/usr/bin/env bash
set -eo pipefail

. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-release-platforms.sh"

for PLATFORM in "${RELEASE_PLATFORMS[@]}"
do
  echo "*** Building release binary for $PLATFORM ***"
  if [[ "$PLATFORM" =~ "-apple-darwin" ]]; then
    CC="o64-clang" CXX="o64-clang++" LIBZ_SYS_STATIC="1" cargo build --release "--target=$PLATFORM"
  elif [[ "$PLATFORM" == "aarch64-unknown-linux-musl" ]]; then
    RUSTFLAGS="-C linker=aarch64-linux-gnu-gcc" cargo build --release "--target=$PLATFORM"
  else
    cargo build --release "--target=$PLATFORM"
  fi
done
