#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# cargo clean

# smoelius: Allow `iter-without-into-iter` until the following issue is resolved:
# https://github.com/bitflags/bitflags/issues/379

cargo +nightly clippy --all-targets -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -A clippy::cognitive-complexity \
    -A clippy::collection-is-never-read \
    -A clippy::items-after-test-module \
    -A clippy::iter-without-into-iter \
    -A clippy::let-underscore-untyped \
    -A clippy::missing-const-for-fn \
    -A clippy::missing-errors-doc \
    -A clippy::missing-panics-doc \
    -A clippy::option-if-let-else \
    -A clippy::redundant-pub-crate \
    -A clippy::type-repetition-in-bounds
