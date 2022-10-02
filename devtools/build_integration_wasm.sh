#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# This file is used for local testing. Once you update Rust files, run this to prepare them for the
# integration tests. Then you can run the integration tests.

# go to root dir regardless of where it was run
SCRIPT_DIR="$(realpath "$(dirname "$0")")"
cd "${SCRIPT_DIR}/.."

# compile all contracts
for C in ./contracts/*/
do
  echo "Compiling $(basename "$C")..."
  (cd "$C" && RUSTFLAGS='-C link-arg=-s' cargo build --release --target wasm32-unknown-unknown --locked)
done

# move them to the internal dir inside tests
mkdir -p ./tests/internal
cp ./target/wasm32-unknown-unknown/release/*.wasm ./tests/internal

ls -l ./tests/internal
