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

exit "$fail"
