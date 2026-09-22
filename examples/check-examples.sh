#!/bin/sh
set -e

cd examples
for i in *.rs; do
    cargo check --example ${i%.rs}
done
