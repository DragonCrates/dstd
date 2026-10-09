#!/bin/sh
set -e

cargo test

cd examples
./check-examples.sh
cd ..

cd tests
./run-tests.sh
cd ..

WIN_NO_RUN=0
if ! x86_64-w64-mingw32-gcc --version >/dev/null; then
    echo "Mingw not available - skipping windows tests"
    WIN_NO_RUN=1
elif ! wine --version >/dev/null; then
    echo "Wine not available - skipping windows tests"
    WIN_NO_RUN=1
fi

if [ "$WIN_NO_RUN" = 0 ]; then
    ./wincargo test
else
    ./wincargo check
fi

cd examples
./win-check-examples.sh
cd ..

if [ "$WIN_NO_RUN" = 0 ]; then
    cd tests
    ./win-run-tests.sh
    cd ..
fi
