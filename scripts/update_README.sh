#! /bin/bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

cd "$(dirname "$0")"/..

USAGE="$(mktemp)"
ARGS="$(mktemp)"
OPTIONS="$(mktemp)"
README="$(mktemp)"

cargo run -p cargo-test-fuzz -- test-fuzz --help |
sed -n '/^Usage:/p' |
sed 's/^Usage://' |
cat > "$USAGE"

cargo run -p cargo-test-fuzz -- test-fuzz --help |
sed -n '/^Arguments:/,/^Options:/p' |
tail -n +2 |
head -n -2 |
cat > "$ARGS"

cargo run -p cargo-test-fuzz -- test-fuzz --help |
sed -n '/^Options:/,$ p' |
tail -n +2 |
cat > "$OPTIONS"

ENABLE=

# smoelius: File to `cat` the next time '```' is encountered.
NEXT=

IFS=''
cat README.md |
while read -r X; do
    if [[ -z "$ENABLE" ]]; then
        # shellcheck disable=SC2016
        if [[ "$X" = '### `cargo test-fuzz` command' ]]; then
            ENABLE=1
        fi
        echo "$X"
        continue
    fi

    if [[ "$NEXT" = '/dev/zero' ]]; then
        if [[ "$X" = '```' ]]; then
            NEXT=
        else
            continue
        fi
    fi

    echo "$X"

    if [[ "$X" = '#### Usage' ]]; then
        NEXT='USAGE'
    elif [[ "$X" = '#### Arguments' ]]; then
        NEXT='ARGS'
    elif [[ "$X" = '#### Options' ]]; then
        NEXT='OPTIONS'
    elif [[ "$X" = '```' ]]; then
        # smoelius: It should not be possible to get here with "$NEXT" = '/dev/zero'.
        if [[ -n "$NEXT" ]]; then
            cat "${!NEXT}"
            NEXT='/dev/zero'
        fi
    fi
done |
cat > "$README"

mv "$README" README.md
