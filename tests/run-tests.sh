#!/bin/sh
set -e

cd examples
for i in *.rs; do
    cargo run --example ${i%.rs}
done
