#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

cargo clean

TMP="$(mktemp)"

cargo clippy --workspace --tests --message-format=json -- \
    -W clippy::style \
    -W clippy::complexity \
    -W clippy::perf \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
|
jq -r 'select(.reason == "compiler-message") | .message | select(.code != null) | .code | .code' |
sort -u |
cat > "$TMP"

diff -s "$(dirname "$0")"/allow.txt "$TMP"
