#! /bin/bash

set -euo pipefail

set -x

cargo test --no-run

cargo test-fuzz --no-run

cargo test-fuzz --no-run --persistent

echo OK
