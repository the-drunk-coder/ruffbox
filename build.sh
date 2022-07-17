#!/bin/sh -e

echo "compile sampler to wasm"
cd ruffbox-synth
cargo build --target wasm32-unknown-unknown --release

echo "copy"
cp target/wasm32-unknown-unknown/release/ruffbox_synth.wasm ../wasm/
cd ..

echo "compile scheduler to wasm"
cd ruffbox-pattern
wasm-pack build --out-dir ../js/pkg --target no-modules

echo "finish!"
