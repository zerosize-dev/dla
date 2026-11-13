#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also record `count` administrative actions (default 30000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/audit/*.java
javac -d out -cp out test/audit/*.java
java -ea -cp out audit.AuditServiceTest
if [ "$1" = "run" ]; then
    java -cp out audit.Main "${2:-30000}"
fi
