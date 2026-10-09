#!/bin/sh
set -e

cargo test

cd examples
./check-examples.sh
cd ..

cd tests
./run-tests.sh
cd ..

if ! x86_64-w64-mingw32-gcc --version >/dev/null; then
    echo "Mingw not available - skipping windows check"
    exit
fi

./wincargo test

cd examples
./win-check-examples.sh
cd ..

if ! wine --version >/dev/null; then
    echo "Wine not available - skipping windows check"
else
    cd tests
    ./win-run-tests.sh
    cd ..
fi
