#!/bin/sh
set -e

cd examples
for i in *.rs; do
    cargo build --example ${i%.rs}
done
