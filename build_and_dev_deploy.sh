#!/bin/bash

CONTRACT_NAME=$(echo "${PWD##*/}" | sed 's/-/_/')

cargo build-release

near dev-deploy \
    --wasmFile "./target/wasm32-unknown-unknown/release/${CONTRACT_NAME}.wasm"
