#!/usr/bin/env bash
set -eo pipefail

. "$CIRCLE_WORKING_DIRECTORY/.circleci/bin/get-version-from-binary.sh"

cd "$CIRCLE_WORKING_DIRECTORY/pkg/npm"

sed -i "s/@QLC_VERSION@/$QLC_VERSION/g" package.json

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
