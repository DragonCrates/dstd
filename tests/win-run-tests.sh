#!/bin/sh
set -e

cd examples
for i in *.rs; do
    ../../wincargo run --example ${i%.rs} --release
done
