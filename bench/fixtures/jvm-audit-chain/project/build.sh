#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also send `count` requests (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/gateway/*.java
javac -d out -cp out test/gateway/*.java
java -ea -cp out gateway.GatewayTest
if [ "$1" = "run" ]; then
    java -cp out gateway.Main "${2:-5000}"
fi
