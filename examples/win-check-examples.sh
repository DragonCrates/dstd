#!/bin/sh
set -e

cd examples
for i in *.rs; do
    cargo check --example ${i%.rs} --target x86_64-pc-windows-gnullvm
done
