#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also price `count` quote requests (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/quotes/*.java
javac -d out -cp out test/quotes/*.java
java -ea -cp out quotes.QuoteServiceTest
if [ "$1" = "run" ]; then
    java -cp out quotes.Main "${2:-5000}"
fi
