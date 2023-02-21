#! /bin/bash

# set -x
set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

cargo +nightly clippy --features=test-fuzz/auto_concretize --all-targets --message-format=json -- \
    --force-warn=clippy::all \
    --force-warn=clippy::pedantic \
    --force-warn=clippy::expect_used \
    --force-warn=clippy::unwrap_used \
    --force-warn=clippy::panic \
    > warnings.json

DYLINT_RUSTFLAGS='--deny warnings' cargo dylint --all -- --features=test-fuzz/auto_concretize --all-targets
