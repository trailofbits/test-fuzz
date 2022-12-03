#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# smoelius: This should match `cargo-test-fuzz/tests/third_party.rs`.
LINES_OF_CONTEXT=2

DIR="$(dirname "$(realpath "$0")")/.."

cd "$DIR"

paste <(jq -r .[].url cargo-test-fuzz/third_party.json) <(jq -r .[].patch cargo-test-fuzz/third_party.json) |
while read -r URL PATCH; do
    pushd "$(mktemp -d)"

    git clone --depth 1 "$URL" .

    git apply --reject "$DIR"/cargo-test-fuzz/patches/"$PATCH" || true

    find "$PWD" -name '*.rej'

    git diff --unified="$LINES_OF_CONTEXT" > "$DIR"/cargo-test-fuzz/patches/"$PATCH"

    popd
done
