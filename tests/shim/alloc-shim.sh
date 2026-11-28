#!/usr/bin/env bash
# Builds the allocator shim with the flags `dla harness gen` uses on this host (Linux or
# macOS) and checks the DLA_SHIM_ONLY name match: a program started through a symbolic
# link is matched by its resolved name, and another name writes no record.
#
# Usage: tests/shim/alloc-shim.sh        (from anywhere; needs cc)
# Exit status 0 when every check passes.

set -u

REPO=$(cd "$(dirname "$0")/../.." && pwd)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/dla-alloc-shim.XXXXXX")
trap 'rm -rf "$WORK"' EXIT

fail=0
check() {
  if "$@"; then
    printf 'ok: %s\n' "$label"
  else
    printf 'FAIL: %s\n' "$label"
    fail=1
  fi
}

case "$(uname -s)" in
  Linux)
    lib="$WORK/libdlashim.so"
    cc -std=gnu11 -O2 -g -fPIC -shared -fno-optimize-sibling-calls -o "$lib" \
      "$REPO/harness/native/dla_alloc_shim.c" -ldl -lpthread || exit 1
    preload=LD_PRELOAD ;;
  Darwin)
    lib="$WORK/libdlashim.dylib"
    cc -std=gnu11 -O2 -g -dynamiclib -o "$lib" "$REPO/harness/native/dla_alloc_shim.c" || exit 1
    preload=DYLD_INSERT_LIBRARIES ;;
  *)
    echo "skip: no allocator shim on $(uname -s)"
    exit 0 ;;
esac

cat > "$WORK/prog.c" <<'EOF'
#include <stdlib.h>
int main(void)
{
    for (int i = 0; i < 1000; i++)
        if (malloc(64) == NULL)
            return 1;
    return 0;
}
EOF
mkdir -p "$WORK/.libs"
cc -O0 -g -o "$WORK/.libs/prog-real" "$WORK/prog.c" || exit 1
ln -s .libs/prog-real "$WORK/prog"

run() { # $1 = DLA_SHIM_ONLY, $2 = output file
  # Exported in a subshell, not through env(1): macOS drops DYLD_* when it starts a
  # protected system binary.
  (
    export "$preload=$lib" DLA_HARNESS_OUT="$2" DLA_SHIM_ONLY="$1"
    "$WORK/prog"
  )
}

run prog-real "$WORK/real.jsonl"
label="a link is matched by the resolved name and writes a record"
check test -s "$WORK/real.jsonl"
label="the record names the resolved executable"
check grep -q 'prog-real' "$WORK/real.jsonl"

run another "$WORK/other.jsonl"
label="another name writes no record"
check test ! -s "$WORK/other.jsonl"

# A snapshot holds state that exit() frees before the destructor runs. The program keeps a 1 MB
# buffer that an atexit handler frees (what a thread-local or a static destructor does); with
# "snap" it calls the exported snapshot first, found by name at run time.
cat > "$WORK/held.c" <<'EOF'
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
static char *held;
static void release(void) { free(held); }
int main(int argc, char **argv)
{
    held = malloc(1000000);
    if (held == NULL)
        return 1;
    memset(held, 1, 1000000);
    atexit(release);
    if (argc > 1 && strcmp(argv[1], "snap") == 0) {
        void (*snap)(uint64_t) = (void (*)(uint64_t))dlsym(RTLD_DEFAULT, "dla_shim_snapshot");
        if (snap != NULL)
            snap(42);
    }
    return 0;
}
EOF
cc -O0 -g -o "$WORK/held" "$WORK/held.c" -ldl || exit 1

held_run() { # $1 = argument, $2 = output file
  (
    export "$preload=$lib" DLA_HARNESS_OUT="$2" DLA_SHIM_ONLY=held
    "$WORK/held" "$1"
  )
}
live_of() { sed -n 's/.*"native.live_bytes":\([0-9]*\).*/\1/p' "$1" | head -1; }

held_run none "$WORK/exit.jsonl"
label="without a snapshot the exit record has lost the state freed at exit"
check test "$(live_of "$WORK/exit.jsonl")" -lt 100000
label="without a snapshot there is one record, marked exit"
check test "$(wc -l < "$WORK/exit.jsonl")" -eq 1
check grep -q '"at":"exit"' "$WORK/exit.jsonl"

held_run snap "$WORK/snap.jsonl"
label="a snapshot holds the live state"
check test "$(live_of "$WORK/snap.jsonl")" -ge 1000000
label="a snapshot is the only record of the process, marked snapshot, with the driver's x"
check test "$(wc -l < "$WORK/snap.jsonl")" -eq 1
check grep -q '"at":"snapshot"' "$WORK/snap.jsonl"
check grep -q '"phase":"exit"' "$WORK/snap.jsonl"
check grep -q '"x":42,' "$WORK/snap.jsonl"

(
  export DLA_SHIM_DISABLE=1
  held_run snap "$WORK/off.jsonl"
)
label="a snapshot call is a no-op when the shim is disabled"
check test ! -s "$WORK/off.jsonl"

exit "$fail"
