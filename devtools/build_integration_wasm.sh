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
  (cd "$C" && RUSTFLAGS='-C link-arg=-s' cargo build --lib --release --target wasm32-unknown-unknown --locked)
done

# move them to the internal dir inside tests
mkdir -p ./tests/internal
cp ./target/wasm32-unknown-unknown/release/*.wasm ./tests/internal

ls -l ./tests/internal

#!/bin/bash

## Compiles an optimizes the local contracts for testing with
## ts-relayer.

# set -o errexit -o nounset -o pipefail
# command -v shellcheck >/dev/null && shellcheck "$0"

# cd "$(git rev-parse --show-toplevel)"

# docker run --rm -v "$(pwd)":/code --platform linux/amd64 \
# 	--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
# 	--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
# 	cosmwasm/workspace-optimizer:0.12.8

# mkdir -p ./tests/internal
# cp ./artifacts/*.wasm ./tests/internal

# echo "done. avaliable wasm blobs:"
# ls ./tests/internal
