#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

URLS=(
    https://github.com/paritytech/substrate
    https://github.com/solana-labs/solana
    https://github.com/substrate-developer-hub/substrate-node-template
)

PATCHES=(
    substrate_client_transaction_pool.patch
    solana.patch
    substrate_node_template.patch
)

# smoelius: This should match `cargo-test-fuzz/tests/third_party.rs`.
LINES_OF_CONTEXT=2

DIR="$(dirname "$(realpath "$0")")/.."

cd "$DIR"

N=${#URLS[@]}

for (( I=0; I<$N; I++ )) {
    URL=${URLS[$I]}
    PATCH=${PATCHES[$I]}

    pushd "$(mktemp -d)"

    git clone "$URL" .

    git apply --reject "$DIR"/cargo-test-fuzz/patches/"$PATCH" || true

    find "$PWD" -name '*.rej'

    git diff --unified="$LINES_OF_CONTEXT" > "$DIR"/cargo-test-fuzz/patches/"$PATCH"

    popd
}
