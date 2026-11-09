#!/bin/sh
# Compiles the sources, runs the unit tests, and optionally the load driver.
#   ./build.sh              compile and run the unit tests
#   ./build.sh run [count]  also render `count` invoices (default 5000)
set -e
cd "$(dirname "$0")"
rm -rf out
mkdir -p out
javac -d out src/billing/*.java
javac -d out -cp out test/billing/*.java
java -ea -cp out billing.InvoiceServiceTest
if [ "$1" = "run" ]; then
    java -cp out billing.Main "${2:-5000}"
fi
