#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# smoelius: This should match `third-party/tests/third_party.rs`.
LINES_OF_CONTEXT=2

DIR="$(dirname "$(realpath "$0")")/.."

cd "$DIR"

paste <(jq -r .[].url third-party/third_party.json) <(jq -r .[].rev third-party/third_party.json) <(jq -r .[].patch third-party/third_party.json) |
while read -r URL REV_OLD PATCH; do
    pushd "$(mktemp -d)"

    git clone --depth 1 "$URL" .

    git apply --reject "$DIR"/third-party/patches/"$PATCH" || true

    find "$PWD" -name '*.rej'

    git diff --unified="$LINES_OF_CONTEXT" > "$DIR"/third-party/patches/"$PATCH"

    # smoelius: Update third_party.json with the revision that was just diffed against.

    REV_NEW="$(git rev-parse HEAD)"

    sed -i "s/\"$REV_OLD\"/\"$REV_NEW\"/" "$DIR"/third-party/third_party.json

    popd
done
