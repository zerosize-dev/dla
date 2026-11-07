#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also replay `count` requests (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/shop/*.java
javac -d out -cp out test/shop/*.java
java -ea -cp out shop.OrderServiceTest
if [ "$1" = "run" ]; then
    java -cp out shop.Main "${2:-5000}"
fi
