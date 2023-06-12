#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

cargo license |
while read -r X; do
    echo "$X"
    if [[ "$X" = 'AGPL-3.0 WITH mif-exception (5): cargo-test-fuzz, test-fuzz, test-fuzz-internal, test-fuzz-macro, test-fuzz-runtime' ]]; then
        continue
    fi
    echo "$X" | grep -o '^[^:]*' | grep -w 'Apache\|BSD-3-Clause\|ISC\|MIT\|N/A' >/dev/null
done
