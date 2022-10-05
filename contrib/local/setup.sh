#!/bin/bash

DENOM="${DENOM:=uxprt}"
CHAIN_DATA_DIR="${CHAIN_DATA_DIR:=.persistenceCore}"
CHAIN="${CHAIN:=juno}"

set -eu

echo "Setting up osmosis chain....."
CHAIN_BIN="$CHAIN"d
VALIDATOR_CONFIG="../../k8s/base/$CHAIN/configs/validators.json"
KEYS_CONFIG="../../k8s/base/$CHAIN/configs/keys.json"
# Add keys to keyringg
jq -r ".genesis[0].mnemonic" $VALIDATOR_CONFIG | $CHAIN_BIN keys add $(jq -r ".genesis[0].name" $VALIDATOR_CONFIG) --recover --keyring-backend="test" --output json | jq
for ((i=0; i<$(jq -r '.validators | length' $VALIDATOR_CONFIG); i++))
do
  jq -r ".validators[$i].mnemonic" $VALIDATOR_CONFIG | $CHAIN_BIN keys add $(jq -r ".validators[$i].name" $VALIDATOR_CONFIG) --recover --keyring-backend="test" --output json | jq
done
for ((i=0; i<$(jq -r '.keys | length' $KEYS_CONFIG); i++))
do
  jq -r ".keys[$i].mnemonic" $KEYS_CONFIG | $CHAIN_BIN keys add $(jq -r ".keys[$i].name" $KEYS_CONFIG) --recover --keyring-backend="test" --output json | jq
done
