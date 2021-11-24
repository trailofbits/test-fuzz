#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

URLS=(
    https://github.com/substrate-developer-hub/substrate-node-template
    https://github.com/paritytech/substrate
)

PATCHES=(
    substrate_node_template.patch
    substrate_client_transaction_pool.patch
)

# smoelius: This should match `cargo-test-fuzz/tests/substrate.rs`.
LINES_OF_CONTEXT=2

DIR="$(dirname "$(realpath "$0")")/.."

cd "$DIR"

N=${#URLS[@]}

for (( I=0; I<$N; I++ )) {
    URL=${URLS[$I]}
    PATCH=${PATCHES[$I]}

    pushd "$(mktemp -d)"

    git clone "$URL" .

    git apply "$DIR"/cargo-test-fuzz/patches/"$PATCH"

    git diff --unified="$LINES_OF_CONTEXT" > "$DIR"/cargo-test-fuzz/patches/"$PATCH"

    popd
}
