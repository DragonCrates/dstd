#!/bin/sh

cargo clean

cd examples
cargo clean

cd test-example
cargo clean
cd ..

cd custom-randomstate
cargo clean
cd ../..

cd tests
cargo clean
