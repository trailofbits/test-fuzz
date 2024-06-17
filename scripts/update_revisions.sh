#! /bin/bash

# smoelius: This script is currently unused.

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

DIR="$(dirname "$(realpath "$0")")/.."

cd "$DIR"

paste <(jq -r .[].url third-party/third_party.json) <(jq -r .[].rev third-party/third_party.json) |
while read -r URL REV_OLD; do
    pushd "$(mktemp -d)"

    git clone --depth 1 "$URL" .

    REV_NEW="$(git rev-parse HEAD)"

    sed -i "s/\"$REV_OLD\"/\"$REV_NEW\"/" "$DIR"/third-party/third_party.json

    popd
done
