#!/bin/sh

echo Dstd
cargo clean

echo Examples
cd examples
cargo clean

echo Examples/test-example
cd test-example
cargo clean
cd ..

echo Examples/custom-randomstate
cd custom-randomstate
cargo clean
cd ../..

echo Tests
cd tests
cargo clean
