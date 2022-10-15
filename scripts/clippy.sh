#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

# cargo clean

cargo clippy --workspace --all-targets -- \
    -D warnings \
    -W clippy::pedantic \
    -W clippy::nursery \
    -W clippy::cargo \
    -A clippy::cargo-common-metadata \
    -A clippy::cognitive-complexity \
    -A clippy::if-not-else \
    -A clippy::missing-const-for-fn \
    -A clippy::missing-errors-doc \
    -A clippy::missing-panics-doc \
    -A clippy::redundant-pub-crate \
    -A clippy::struct-excessive-bools \
    -A clippy::type-repetition-in-bounds \
    -A clippy::too-many-lines
