#! /bin/bash

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "$0: expect one argument: version" >&2
    exit 1
fi

set -x

SCRIPTS="$(dirname "$(realpath "$0")")"
WORKSPACE="$(realpath "$SCRIPTS"/..)"

cd "$WORKSPACE"

VERSION="version = \"$1\""

find . -name Cargo.toml -exec sed -i "{
s/^version = \"[^\"]*\"$/$VERSION/
}" {} \;

REQ="${VERSION/\"/\"=}"

sed -i "/^test-fuzz/{
s/^\(.*\)\<version = \"[^\"]*\"\(.*\)$/\1$REQ\2/
}" Cargo.toml

sed -i "/\<package = \"test-fuzz/{
s/^\(.*\)\<version = \"[^\"]*\"\(.*\)$/\1$REQ\2/
}" Cargo.toml
