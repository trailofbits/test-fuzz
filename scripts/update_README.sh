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
sed -n '/^USAGE:/,/^ARGS:/p' |
tail -n +2 |
head -n -2 |
cat > "$USAGE"

cargo run -p cargo-test-fuzz -- test-fuzz --help |
sed -n '/^ARGS:/,/^OPTIONS:/p' |
tail -n +2 |
head -n -2 |
cat > "$ARGS"

cargo run -p cargo-test-fuzz -- test-fuzz --help |
sed -n '/^OPTIONS:/,$ p' |
tail -n +2 |
cat > "$OPTIONS"

NEXT=

IFS=''
cat README.md |
while read -r X; do
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
    elif [[ "$X" = '#### Args' ]]; then
        NEXT='ARGS'
    elif [[ "$X" = '#### Options' ]]; then
        NEXT='OPTIONS'
    elif [[ "$X" = '```' ]]; then
        # smoelius: It should not be possible to get here with "$NEXT" = 'dev/zero'.
        if [[ -n "$NEXT" ]]; then
            cat "${!NEXT}"
            NEXT='/dev/zero'
        fi
    fi
done |
cat > "$README"

mv "$README" README.md
