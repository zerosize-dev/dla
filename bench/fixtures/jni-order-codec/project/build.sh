#!/bin/sh
# Builds the native codec and the Kotlin service, runs the tests, and optionally the replay driver.
#   ./build.sh              build and run the tests
#   ./build.sh run [count]  also replay `count` orders (default 10000)
set -e
cd "$(dirname "$0")"

(cd native && cargo build --release --offline && cargo test --release --offline -q)

# Kotlin: use kotlinc when installed, otherwise the embeddable compiler from the local Maven cache.
M=${M2:-$HOME/.m2/repository}
K=$M/org/jetbrains/kotlin
STD=$K/kotlin-stdlib/2.0.21/kotlin-stdlib-2.0.21.jar
mkdir -p out
if command -v kotlinc >/dev/null 2>&1; then
    # The stdlib next to kotlinc (the compiler and the program then use one version);
    # the cached jar when this install has no lib/kotlin-stdlib.jar.
    KC=$(command -v kotlinc)
    KC=$(readlink -f "$KC" 2>/dev/null || echo "$KC")
    KLIB=$(dirname "$KC")/../lib/kotlin-stdlib.jar
    if [ -f "$KLIB" ]; then STD=$KLIB; fi
    kotlinc -d out kotlin/shop/*.kt
else
    CP=$K/kotlin-compiler-embeddable/2.0.21/kotlin-compiler-embeddable-2.0.21.jar:$STD
    CP=$CP:$K/kotlin-script-runtime/2.0.21/kotlin-script-runtime-2.0.21.jar
    CP=$CP:$K/kotlin-reflect/1.6.10/kotlin-reflect-1.6.10.jar
    CP=$CP:$K/kotlin-daemon-embeddable/2.0.21/kotlin-daemon-embeddable-2.0.21.jar
    CP=$CP:$M/org/jetbrains/intellij/deps/trove4j/1.0.20200330/trove4j-1.0.20200330.jar
    CP=$CP:$M/org/jetbrains/annotations/13.0/annotations-13.0.jar
    CP=$CP:$M/org/jetbrains/kotlinx/kotlinx-coroutines-core-jvm/1.6.4/kotlinx-coroutines-core-jvm-1.6.4.jar
    java -cp "$CP" org.jetbrains.kotlin.cli.jvm.K2JVMCompiler -no-stdlib -cp "$STD" -d out kotlin/shop/*.kt
fi

RUN_CP="out:$STD"
LIB="-Djava.library.path=native/target/release"
java -ea $LIB -cp "$RUN_CP" shop.codec.CodecTestKt
if [ "$1" = "run" ]; then
    exec java -Xmx256m $LIB -cp "$RUN_CP" shop.codec.MainKt "${2:-10000}"
fi
