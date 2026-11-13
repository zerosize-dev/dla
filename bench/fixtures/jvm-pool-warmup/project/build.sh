#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also run `count` export jobs (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/exports/*.java
javac -d out -cp out test/exports/*.java
java -ea -cp out exports.ExporterTest
if [ "$1" = "run" ]; then
    java -cp out exports.Main "${2:-5000}"
fi
