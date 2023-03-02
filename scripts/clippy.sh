#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# cargo clean

cargo +nightly clippy --features=test-fuzz/auto_concretize --all-targets -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -A clippy::cognitive-complexity \
    -A clippy::let-underscore-untyped \
    -A clippy::missing-const-for-fn \
    -A clippy::missing-errors-doc \
    -A clippy::missing-panics-doc \
    -A clippy::option-if-let-else \
    -A clippy::redundant-pub-crate \
    -A clippy::type-repetition-in-bounds
