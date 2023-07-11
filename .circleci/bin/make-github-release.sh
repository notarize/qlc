#!/usr/bin/env bash
set -eo pipefail

ARCHIVES_DIR="$CIRCLE_WORKING_DIRECTORY/archives"
mkdir -p "$ARCHIVES_DIR"

. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-version-from-binary.sh"
. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-release-platforms.sh"

for PLATFORM in "${RELEASE_PLATFORMS[@]}"
do
  cd "$CIRCLE_WORKING_DIRECTORY/target/$PLATFORM/release"
  tar czf "qlc-$QLC_VERSION-$PLATFORM.tar.gz" qlc
  mv "qlc-$QLC_VERSION-$PLATFORM.tar.gz" "$ARCHIVES_DIR"
done

echo "*** Making release out of $ARCHIVES_DIR ***"
ls -l "$ARCHIVES_DIR"

GITHUB_RELEASE_ARGS=(
  -n "v$QLC_VERSION"
  -c "$CIRCLE_SHA1"
  -delete
)
if [[ $QLC_VERSION =~ "-alpha" ]] || [[ $QLC_VERSION =~ "-beta" ]]; then
  GITHUB_RELEASE_ARGS+=(-prerelease)
fi
ghr -t "$GITHUB_TOKEN" "${GITHUB_RELEASE_ARGS[@]}" "$QLC_VERSION" "$ARCHIVES_DIR"
