#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also replay `count` logins (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/portal/*.java
javac -d out -cp out test/portal/*.java
java -ea -cp out portal.SessionServiceTest
if [ "$1" = "run" ]; then
    java -cp out portal.Main "${2:-5000}"
fi
