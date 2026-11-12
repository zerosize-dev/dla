#!/bin/sh
# Build step of a JUnit repeat harness (run by `dla harness run`, never by `dla harness gen`).
#   dla-junit-build.sh <gradle|maven> <tool> <driver dir> <agent jar> <path separator>
# <tool> is the build tool executable, or the project's wrapper script. The script
#   1. compiles the project's main and test code and writes its test class path to <driver>/cp.txt
#      (Gradle: an init script; Maven: dependency:build-classpath plus target/test-classes and
#      target/classes), adding junit-platform-launcher of the engine's version when it is missing,
#   2. compiles <driver>/DlaJUnitLoop.java against it,
#   3. writes <driver>/run.args, the Java argument file `java @run.args DlaJUnitLoop` runs with.
# Offline is tried first; the online resolution is the fallback.
set -eu
kind=$1; tool=$2; d=$3; jar=$4; sep=$5

run_tool() {
    if [ -f "$tool" ] && [ ! -x "$tool" ]; then sh "$tool" "$@"; else "$tool" "$@"; fi
}

# Printed between the offline attempt and the online one: output above it is the offline attempt's.
retry() { echo "DLA-BUILD the offline attempt failed; trying online" >&2; }

if [ "$kind" = gradle ]; then
    init="$d/dla-junit.init.gradle"
    # The init script reads the project model in the task action, which the configuration cache
    # (org.gradle.configuration-cache=true in gradle.properties) cannot store: turn it off here.
    run_tool --no-daemon -q --offline -Dorg.gradle.configuration-cache=false -Dorg.gradle.unsafe.configuration-cache=false --init-script "$init" "-Pdla.cpfile=$d/cp.txt" dlaPrintTestClasspath ||
        {
            retry
            run_tool --no-daemon -q -Dorg.gradle.configuration-cache=false -Dorg.gradle.unsafe.configuration-cache=false --init-script "$init" "-Pdla.cpfile=$d/cp.txt" dlaPrintTestClasspath
        }
    cp=$(cat "$d/cp.txt")
else
    mvn_cp="$d/cp.mvn.txt"
    run_tool -q -B -o test-compile dependency:build-classpath -Dmdep.includeScope=test "-Dmdep.outputFile=$mvn_cp" ||
        {
            retry
            run_tool -q -B test-compile dependency:build-classpath -Dmdep.includeScope=test "-Dmdep.outputFile=$mvn_cp"
        }
    # Relative to the project directory, where the run starts too (an absolute path from a POSIX
    # shell, /c/..., is not one a native Windows java resolves). Maven's own paths may carry
    # backslashes on Windows: forward slashes work there as well, and the repository prefix below
    # is cut at a forward slash.
    cp="target/test-classes${sep}target/classes${sep}$(tr '\\' / < "$mvn_cp")"
    case "$cp" in
    *junit-platform-launcher-[0-9]*) ;;
    *)
        engine=$(printf '%s' "$cp" | tr "$sep" '\n' | grep 'junit-platform-engine-[0-9][^/]*\.jar$' | head -n 1 || true)
        if [ -z "$engine" ]; then
            echo "DLA: no junit-platform-engine on the test class path: the tests are not JUnit 5" >&2
            exit 1
        fi
        ver=$(basename "$engine" .jar)
        ver=${ver#junit-platform-engine-}
        repo=${engine%/org/junit/platform/junit-platform-engine/*}
        launcher="$repo/org/junit/platform/junit-platform-launcher/$ver/junit-platform-launcher-$ver.jar"
        if [ ! -f "$launcher" ]; then
            run_tool -q -B -o dependency:get "-Dartifact=org.junit.platform:junit-platform-launcher:$ver" ||
                {
                    retry
                    run_tool -q -B dependency:get "-Dartifact=org.junit.platform:junit-platform-launcher:$ver"
                }
        fi
        if [ ! -f "$launcher" ]; then
            echo "DLA: junit-platform-launcher $ver is not in $repo" >&2
            exit 1
        fi
        cp="$cp$sep$launcher"
        ;;
    esac
    printf '%s' "$cp" > "$d/cp.txt"
fi

mkdir -p "$d/classes"
javac -proc:none -nowarn -cp "$cp$sep$jar" -d "$d/classes" "$d/DlaJUnitLoop.java"
full="$cp$sep$jar$sep$d/classes"
escaped=$(printf '%s' "$full" | sed 's/\\/\\\\/g; s/"/\\"/g')
printf -- '-cp\n"%s"\n' "$escaped" > "$d/run.args"
