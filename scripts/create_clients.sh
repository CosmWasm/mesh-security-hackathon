#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

SCRIPT_DIR="$(realpath "$(dirname "$0")")"
cd "${SCRIPT_DIR}/../tests"

#Create client for mesh-consumer
cosmwasm-ts-codegen generate \
 --plugin client \
 --schema ls ../contracts/mesh-consumer/schema \
 --out ./src/bindings \
 --name MeshConsumer \
 --no-bundle

 #Create client for mesh-vault
cosmwasm-ts-codegen generate \
 --plugin client \
 --schema ls ../contracts/mesh-vault/schema \
 --out ./src/bindings \
 --name MeshVault \
 --no-bundle

 #Create client for mesh-provider
cosmwasm-ts-codegen generate \
 --plugin client \
 --schema ls ../contracts/mesh-provider/schema \
 --out ./src/bindings \
 --name MeshProvider \
 --no-bundle

#Create client for mesh-slasher
cosmwasm-ts-codegen generate \
 --plugin client \
 --schema ls ../contracts/mesh-slasher/schema \
 --out ./src/bindings \
 --name MeshSlasher \
 --no-bundle

  #Create client for meeta-staking
cosmwasm-ts-codegen generate \
 --plugin client \
 --schema ls ../contracts/meta-staking/schema \
 --out ./src/bindings \
 --name MetaStaking \
 --no-bundle
