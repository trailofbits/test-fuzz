#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# cargo clean

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
while read X; do
    if ! grep "^$X$" "$(dirname "$0")"/allow.txt >/dev/null; then
        echo "$X"
    fi
done |
cat > "$TMP"

if [[ ! -s "$TMP" ]]; then
    exit
fi

cargo clean

cat "$TMP" |
while read X; do
    echo -D "$X"
done |
xargs cargo clippy --workspace --tests --
