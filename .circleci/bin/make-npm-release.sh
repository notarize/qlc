#!/usr/bin/env bash
set -eo pipefail

. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-version-from-binary.sh"
. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-release-platforms.sh"

cd "$CIRCLE_WORKING_DIRECTORY/pkg/npm"

sed -i "s/@QLC_VERSION@/$QLC_VERSION/g" package.json
for PLATFORM in "${RELEASE_PLATFORMS[@]}"
do
  CHECKSUM_LINE="$(grep "qlc-$QLC_VERSION-$PLATFORM.tar.gz" "$CIRCLE_WORKING_DIRECTORY/archives/checksums.txt")"
  CHECKSUM_LINE_SPLIT=($CHECKSUM_LINE)
  sed -i "s/@${PLATFORM}_CHECKSUM@/${CHECKSUM_LINE_SPLIT[0]}/g" package.json
done

NPM_RELEASE_ARGS=(
  --new-version "$QLC_VERSION"
  --access public
)
if [[ $QLC_VERSION =~ "-alpha" ]]; then
  NPM_RELEASE_ARGS+=(--tag alpha)
elif [[ $QLC_VERSION =~ "-beta" ]]; then
  NPM_RELEASE_ARGS+=(--tag beta)
fi
yarn publish "${NPM_RELEASE_ARGS[@]}"
