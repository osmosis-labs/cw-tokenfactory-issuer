#!/usr/bin/env bash
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )

# assuming test1,test2 has been setup to keyring

# create denom
echo "Creating denom ..."
osmosisd tx tokenfactory create-denom uusd --from test1 -y &> /dev/null

# deploy contract
echo "Deploying contract ..."
beaker wasm deploy tokenfactory-issuer --signer-account test1  --raw '{"denom":"factory/osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks/uusd"}' --no-wasm-opt &> /dev/null

CONTRACT_ADDR=$(cat $SCRIPT_DIR/.beaker/state.local.json | jq '.local."tokenfactory-issuer".addresses.default' | sed 's/"//g') 

# setup beforesend listener
echo "Setting Before send listener ..."
osmosisd tx tokenfactory set-beforesend-listener factory/osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks/uusd $CONTRACT_ADDR --from test1 -y &> /dev/null

# transfer admin control
echo "Transfering admin to contract ..."
osmosisd tx tokenfactory change-admin factory/osmo1cyyzpxplxdzkeea7kwsydadg87357qnahakaks/uusd $CONTRACT_ADDR --from test1 -y &> /dev/null

