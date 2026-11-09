#!/bin/sh
# Compiles the host and the extension, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also reload the extension `count` times (default 2000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out/host out/ext
javac -d out/host src/reports/*.java
javac -d out/ext -cp out/host ext/ledger/*.java
javac -d out/host -cp out/host test/reports/*.java
java -ea -cp out/host reports.ExtensionManagerTest out/ext
if [ "$1" = "run" ]; then
    java -cp out/host reports.Main out/ext "${2:-2000}"
fi
