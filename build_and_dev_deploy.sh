#!/bin/bash

cargo build-release

near dev-deploy \
    --wasmFile ./target/wasm32-unknown-unknown/release/near_counter_contract.wasm
