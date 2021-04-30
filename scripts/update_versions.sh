#! /bin/bash

set -euo pipefail

if [[ $# -ne 0 ]]; then
    echo "$0: expect no arguments" >&2
    exit 1
fi

set -x

VERSION="$(grep -m 1 '^version = "[^"]*"$' Cargo.toml)"

find . -name Cargo.toml -exec sed -i "{
s/^version = \"[^\"]*\"$/$VERSION/
}" {} \;

REQ="$(echo "$VERSION" | sed 's/"/"=/')"

find . -name Cargo.toml -exec sed -i "/^test-fuzz/{
s/^\(.*\)\<version = \"[^\"]*\"\(.*\)$/\1$REQ\2/
}" {} \;

find . -name Cargo.toml -exec sed -i "/\<package = \"test-fuzz/{
s/^\(.*\)\<version = \"[^\"]*\"\(.*\)$/\1$REQ\2/
}" {} \;
