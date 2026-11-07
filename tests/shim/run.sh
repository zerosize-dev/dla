#!/usr/bin/env bash
# Self-contained test for hooks/fetch-dla.sh and bin/dla.
#
# Builds a fake release in a temp directory, runs the downloader against it with a
# temp CLAUDE_PLUGIN_DATA, and exercises the shim. Uses target/debug/dla as the
# release payload when it exists, otherwise a tiny shell script.
#
# Usage: tests/shim/run.sh        (from anywhere)
# Exit status 0 when every check passes.

set -u

REPO=$(cd "$(dirname "$0")/../.." && pwd)
FETCH="$REPO/hooks/fetch-dla.sh"
SHIM="$REPO/bin/dla"
# Same mapping as detect_target in bin/dla and hooks/fetch-dla.sh.
host_target() {
  os=$(uname -s 2>/dev/null)
  arch=$(uname -m 2>/dev/null)
  case "$os" in
    Linux)
      case "$arch" in
        x86_64|amd64) printf 'x86_64-unknown-linux-musl' ;;
        aarch64|arm64) printf 'aarch64-unknown-linux-musl' ;;
        *) return 1 ;;
      esac ;;
    Darwin)
      case "$arch" in
        arm64|aarch64) printf 'aarch64-apple-darwin' ;;
        x86_64|amd64) printf 'x86_64-apple-darwin' ;;
        *) return 1 ;;
      esac ;;
    MINGW*|MSYS*|CYGWIN*)
      case "$arch" in
        x86_64|amd64) printf 'x86_64-pc-windows-msvc' ;;
        *) return 1 ;;
      esac ;;
    *) return 1 ;;
  esac
}

ON_WINDOWS=0
HOST_EXT=""
case "$(uname -s 2>/dev/null)" in MINGW*|MSYS*|CYGWIN*) ON_WINDOWS=1; HOST_EXT=".exe" ;; esac
HOST_TARGET=$(host_target) || { echo "unsupported host $(uname -s)/$(uname -m)"; exit 1; }

# DLA_TARGET is honoured when the caller sets it (to exercise the override for
# another target); otherwise it stays unset so the uname branch runs on this OS.
TARGET_FROM_ENV=0
if [ -n "${DLA_TARGET:-}" ]; then
  TARGET=$DLA_TARGET
  TARGET_FROM_ENV=1
else
  TARGET=$HOST_TARGET
  unset DLA_TARGET
fi
EXT=""
case "$TARGET" in *windows*) EXT=".exe" ;; esac
if [ "$TARGET_FROM_ENV" = 1 ]; then echo "target: $TARGET (DLA_TARGET set)"; else echo "target: $TARGET (DLA_TARGET unset)"; fi

WORK=$(mktemp -d "${TMPDIR:-/tmp}/dla-shim-test.XXXXXX") || exit 1
trap 'rm -rf "$WORK"' EXIT

PASS=0
FAIL=0
ok() { PASS=$((PASS + 1)); printf 'ok   %s\n' "$1"; }
bad() { FAIL=$((FAIL + 1)); printf 'FAIL %s\n' "$1"; }
check() { # check <description> <command...>
  desc=$1; shift
  if "$@" >/dev/null 2>&1; then ok "$desc"; else bad "$desc"; fi
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then sha256sum "$1" | awk '{print $1}'; else shasum -a 256 "$1" | awk '{print $1}'; fi
}

VERSION=$(sed -n 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$REPO/.claude-plugin/plugin.json" | head -n 1)
[ -n "$VERSION" ] || { echo "cannot read version from plugin.json"; exit 1; }

# --- fake release -----------------------------------------------------------
REL="$WORK/release"
PAYLOAD="$WORK/payload"
mkdir -p "$REL" "$PAYLOAD"
if [ -f "$REPO/target/debug/dla$HOST_EXT" ] && [ "$EXT" = "$HOST_EXT" ] && [ "$TARGET" = "$HOST_TARGET" ]; then
  cp "$REPO/target/debug/dla$HOST_EXT" "$PAYLOAD/dla$EXT"
  echo "payload: real binary target/debug/dla$HOST_EXT"
else
  cat > "$PAYLOAD/dla$EXT" <<EOF
#!/bin/sh
if [ "\${1:-}" = "--version" ]; then echo "dla $VERSION"; exit 0; fi
echo "args: \$#"
for a in "\$@"; do echo "arg: \$a"; done
exit \${DLA_FAKE_EXIT:-0}
EOF
  chmod +x "$PAYLOAD/dla$EXT"
  echo "payload: fake shell script"
fi
ASSET="dla-$TARGET.tar.gz"
tar -czf "$REL/$ASSET" -C "$PAYLOAD" "dla$EXT"
printf '%s  %s\n' "$(sha256_of "$REL/$ASSET")" "$ASSET" > "$REL/$ASSET.sha256"
cp "$REL/$ASSET.sha256" "$REL/SHA256SUMS"

HOME_DIR="$WORK/home"
mkdir -p "$HOME_DIR"
export HOME="$HOME_DIR"
unset DLA_BIN CLAUDE_CONFIG_DIR CLAUDE_CODE_PLUGIN_CACHE_DIR CLAUDE_PLUGIN_DATA DLA_RELEASE_BASE_URL

run_fetch() { # run_fetch <data dir> <base url>; captures stdout in $OUT, status in $RC
  OUT=$(CLAUDE_PLUGIN_ROOT="$REPO" CLAUDE_PLUGIN_DATA="$1" DLA_RELEASE_BASE_URL="$2" bash "$FETCH" 2>"$WORK/stderr")
  RC=$?
}

# --- 1. fresh install from a directory path ---------------------------------
DATA1="$WORK/data1"
DEST1="$DATA1/bin/dla-$VERSION-$TARGET$EXT"
run_fetch "$DATA1" "$REL"
[ "$RC" -eq 0 ] && ok "fresh install exits 0" || bad "fresh install exits 0 (rc=$RC)"
[ -z "$OUT" ] && ok "fresh install prints nothing on stdout" || bad "fresh install stdout not empty: $OUT"
check "installed file exists at the contract path" test -f "$DEST1"
check "installed file is executable" test -x "$DEST1"
check "installed file matches the payload" cmp -s "$DEST1" "$PAYLOAD/dla$EXT"
LEFT=$(find "$DATA1" -name '.fetch.*' | wc -l | tr -d ' ')
[ "$LEFT" = "0" ] && ok "no temporary files left behind" || bad "temporary files left behind ($LEFT)"

# --- 2. second run does not download again ----------------------------------
BEFORE=$(sha256_of "$DEST1")
run_fetch "$DATA1" "$WORK/does-not-exist"
[ "$RC" -eq 0 ] && [ -z "$OUT" ] && ok "second run is silent (no download attempted)" || bad "second run attempted a download: rc=$RC out=$OUT"
[ "$(sha256_of "$DEST1")" = "$BEFORE" ] && ok "second run leaves the binary untouched" || bad "second run changed the binary"

# --- 3. file:// base URL ----------------------------------------------------
DATA2="$WORK/data2"
run_fetch "$DATA2" "file://$REL"
[ "$RC" -eq 0 ] && [ -z "$OUT" ] && [ -f "$DATA2/bin/dla-$VERSION-$TARGET$EXT" ] && ok "file:// base URL installs" || bad "file:// base URL failed: rc=$RC out=$OUT"

# --- 4. corrupted checksum is rejected --------------------------------------
BADREL="$WORK/badrel"
cp -R "$REL" "$BADREL"
printf '%s  %s\n' "0000000000000000000000000000000000000000000000000000000000000000" "$ASSET" > "$BADREL/$ASSET.sha256"
DATA3="$WORK/data3"
DEST3="$DATA3/bin/dla-$VERSION-$TARGET$EXT"
run_fetch "$DATA3" "$BADREL"
[ "$RC" -eq 0 ] && ok "corrupted checksum still exits 0" || bad "corrupted checksum exit code $RC"
[ ! -e "$DEST3" ] && ok "corrupted checksum installs nothing" || bad "corrupted checksum installed a file"
case "$OUT" in *"mismatch"*"$DEST3"*) ok "mismatch message names the manual path" ;; *) bad "mismatch message missing the path: $OUT" ;; esac
LEFT=$(find "$DATA3" -name '.fetch.*' | wc -l | tr -d ' ')
[ "$LEFT" = "0" ] && ok "no temporary files after a rejected download" || bad "temporary files after a rejected download ($LEFT)"

# --- 5. tampered archive with a matching-looking checksum file --------------
TAMPER="$WORK/tamper"
cp -R "$REL" "$TAMPER"
printf 'garbage' >> "$TAMPER/$ASSET"
DATA4="$WORK/data4"
run_fetch "$DATA4" "$TAMPER"
[ "$RC" -eq 0 ] && [ ! -e "$DATA4/bin/dla-$VERSION-$TARGET$EXT" ] && ok "tampered archive is rejected" || bad "tampered archive was installed"

# --- 6. unreachable release --------------------------------------------------
DATA5="$WORK/data5"
run_fetch "$DATA5" "$WORK/nowhere"
[ "$RC" -eq 0 ] && ok "missing release exits 0" || bad "missing release exit code $RC"
case "$OUT" in *"$DATA5/bin/dla-$VERSION-$TARGET$EXT"*) ok "missing release message names the manual path" ;; *) bad "missing release message lacks the path: $OUT" ;; esac

# --- 7. hook never writes into the plugin root or the project ---------------
if [ -z "$(cd "$REPO" && git status --porcelain --ignored 2>/dev/null | grep -E '\.fetch\.|dla-[0-9]+\.[0-9]+\.[0-9]+-' )" ]; then
  ok "nothing written into the repository"
else
  bad "files appeared in the repository"
fi

# --- shim -------------------------------------------------------------------
# A private copy of the plugin root so the repository's own target/ directory
# cannot satisfy the lookup.
PR="$WORK/plugin-root"
mkdir -p "$PR/bin" "$PR/.claude-plugin"
cp "$SHIM" "$PR/bin/dla"
cp "$REPO/.claude-plugin/plugin.json" "$PR/.claude-plugin/plugin.json"
chmod +x "$PR/bin/dla"
PSHIM="$PR/bin/dla"

# 8. nothing installed: exit 127 with the manual path
OUT=$("$PSHIM" --version 2>&1); RC=$?
[ "$RC" -eq 127 ] && ok "shim without a binary exits 127" || bad "shim without a binary exited $RC"
case "$OUT" in *"dla-$VERSION-$TARGET$EXT"*) ok "shim names the file to place by hand" ;; *) bad "shim message lacks the file name: $OUT" ;; esac

# 9. CLAUDE_PLUGIN_DATA set (hook style)
OUT=$(CLAUDE_PLUGIN_DATA="$DATA1" "$PSHIM" --version 2>&1); RC=$?
EXPECT=$("$DEST1" --version 2>&1)
[ "$RC" -eq 0 ] && [ "$OUT" = "$EXPECT" ] && ok "shim runs the installed binary via CLAUDE_PLUGIN_DATA" || bad "shim via CLAUDE_PLUGIN_DATA: rc=$RC out=$OUT"

# 10. Bash-tool style: no plugin vars, data dir found under the config dir
CFG="$WORK/cfg"
mkdir -p "$CFG/plugins/data/dla-zerosize/bin"
cp "$DEST1" "$CFG/plugins/data/dla-zerosize/bin/"
OUT=$(CLAUDE_CONFIG_DIR="$CFG" "$PSHIM" --version 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "$EXPECT" ] && ok "shim finds the data dir under CLAUDE_CONFIG_DIR" || bad "shim via CLAUDE_CONFIG_DIR: rc=$RC out=$OUT"

PLUG="$WORK/plug-cache"
mkdir -p "$PLUG/data/dla-zerosize/bin"
cp "$DEST1" "$PLUG/data/dla-zerosize/bin/"
OUT=$(CLAUDE_CODE_PLUGIN_CACHE_DIR="$PLUG" "$PSHIM" --version 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "$EXPECT" ] && ok "shim honours CLAUDE_CODE_PLUGIN_CACHE_DIR" || bad "shim via CLAUDE_CODE_PLUGIN_CACHE_DIR: rc=$RC out=$OUT"

# 11. HOME-based default location
mkdir -p "$HOME/.claude/plugins/data/dla-zerosize/bin"
cp "$DEST1" "$HOME/.claude/plugins/data/dla-zerosize/bin/"
OUT=$("$PSHIM" --version 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "$EXPECT" ] && ok "shim finds the data dir under \$HOME/.claude" || bad "shim via HOME default: rc=$RC out=$OUT"
rm -rf "$HOME/.claude"

# 12. DLA_BIN wins over everything
FAKE="$WORK/other-dla"
printf '#!/bin/sh\necho override\n' > "$FAKE"; chmod +x "$FAKE"
OUT=$(DLA_BIN="$FAKE" CLAUDE_PLUGIN_DATA="$DATA1" "$PSHIM" 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "override" ] && ok "DLA_BIN takes precedence" || bad "DLA_BIN precedence: rc=$RC out=$OUT"
OUT=$(DLA_BIN="$WORK/missing" "$PSHIM" 2>&1); RC=$?
[ "$RC" -eq 127 ] && ok "bad DLA_BIN exits 127" || bad "bad DLA_BIN exited $RC"

# 13. dev fallback: target/release then target/debug
mkdir -p "$PR/target/debug" "$PR/target/release"
printf '#!/bin/sh\necho debug\n' > "$PR/target/debug/dla$EXT"; chmod +x "$PR/target/debug/dla$EXT"
OUT=$("$PSHIM" 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "debug" ] && ok "dev fallback uses target/debug" || bad "dev fallback debug: rc=$RC out=$OUT"
printf '#!/bin/sh\necho release\n' > "$PR/target/release/dla$EXT"; chmod +x "$PR/target/release/dla$EXT"
OUT=$("$PSHIM" 2>&1); RC=$?
[ "$RC" -eq 0 ] && [ "$OUT" = "release" ] && ok "dev fallback prefers target/release" || bad "dev fallback release: rc=$RC out=$OUT"
OUT=$(CLAUDE_PLUGIN_DATA="$DATA1" "$PSHIM" --version 2>&1)
[ "$OUT" = "$EXPECT" ] && ok "installed binary beats the dev fallback" || bad "installed binary should beat dev fallback: $OUT"
rm -rf "$PR/target"

# 14. arguments and exit status pass through unchanged (fake payload only)
if grep -q 'DLA_FAKE_EXIT' "$DEST1" 2>/dev/null; then
  OUT=$(DLA_FAKE_EXIT=2 CLAUDE_PLUGIN_DATA="$DATA1" "$PSHIM" run "two words" '$x' 2>&1); RC=$?
  EXPECTED_ARGS=$(printf 'args: 3\narg: run\narg: two words\narg: $x')
  [ "$RC" -eq 2 ] && ok "shim propagates the exit status" || bad "exit status not propagated: $RC"
  [ "$OUT" = "$EXPECTED_ARGS" ] && ok "shim passes arguments through verbatim" || bad "arguments mangled: $OUT"
fi

# 15. the shim works through a symlink
if [ "$ON_WINDOWS" = 1 ]; then
  echo "skip shim through a symlink (Git Bash 'ln -s' copies the file, so there is no link to resolve)"
else
  LN="$WORK/link-dla"
  if ln -s "$PSHIM" "$LN" 2>/dev/null; then
    OUT=$(CLAUDE_PLUGIN_DATA="$DATA1" "$LN" --version 2>&1); RC=$?
    [ "$RC" -eq 0 ] && [ "$OUT" = "$EXPECT" ] && ok "shim resolves its plugin root through a symlink" || bad "symlinked shim: rc=$RC out=$OUT"
  else
    bad "could not create a symlink"
  fi
fi

# 15b. DLA_TARGET unset: the uname branch picks the target on this OS
HREL="$REL"
if [ "$TARGET" != "$HOST_TARGET" ]; then
  HREL="$WORK/host-release"
  mkdir -p "$HREL" "$WORK/host-payload"
  printf '#!/bin/sh\necho host\n' > "$WORK/host-payload/dla$HOST_EXT"; chmod +x "$WORK/host-payload/dla$HOST_EXT"
  tar -czf "$HREL/dla-$HOST_TARGET.tar.gz" -C "$WORK/host-payload" "dla$HOST_EXT"
  printf '%s  %s\n' "$(sha256_of "$HREL/dla-$HOST_TARGET.tar.gz")" "dla-$HOST_TARGET.tar.gz" > "$HREL/dla-$HOST_TARGET.tar.gz.sha256"
fi
DATA6="$WORK/data6"
OUT=$(env -u DLA_TARGET CLAUDE_PLUGIN_ROOT="$REPO" CLAUDE_PLUGIN_DATA="$DATA6" DLA_RELEASE_BASE_URL="$HREL" bash "$FETCH" 2>"$WORK/stderr"); RC=$?
[ "$RC" -eq 0 ] && [ -z "$OUT" ] && [ -f "$DATA6/bin/dla-$VERSION-$HOST_TARGET$HOST_EXT" ] && ok "DLA_TARGET unset: hook installs the uname-detected target ($HOST_TARGET)" || bad "DLA_TARGET unset: rc=$RC out=$OUT"
OUT=$(env -u DLA_TARGET CLAUDE_PLUGIN_DATA="$DATA6" "$PSHIM" --version 2>&1); RC=$?
[ "$RC" -eq 0 ] && ok "DLA_TARGET unset: shim finds the uname-detected binary" || bad "DLA_TARGET unset: shim rc=$RC out=$OUT"
rm -rf "$WORK/host-release" "$WORK/host-payload"

# 15c. a data directory with a colon in its name (GNU tar reads "C:/x" as host:file)
if [ "$ON_WINDOWS" = 1 ]; then
  # Claude Code hands hooks a native drive-letter path, not the POSIX /tmp path
  # mktemp gives in Git Bash. Exercise both native spellings: C:/x and C:\x.
  for form in -m -w; do
    mkdir -p "$WORK/native$form"
    NATIVE=$(cygpath "$form" "$WORK/native$form")
    OUT=$(CLAUDE_PLUGIN_ROOT="$REPO" CLAUDE_PLUGIN_DATA="$NATIVE" DLA_RELEASE_BASE_URL="$REL" bash "$FETCH" 2>"$WORK/stderr"); RC=$?
    [ "$RC" -eq 0 ] && [ -z "$OUT" ] && [ -f "$WORK/native$form/bin/dla-$VERSION-$TARGET$EXT" ] && ok "native data dir '$NATIVE' installs" || bad "native data dir '$NATIVE': rc=$RC out=$OUT err=$(cat "$WORK/stderr")"
    OUT=$(CLAUDE_PLUGIN_DATA="$NATIVE" "$SHIM" --version 2>&1); RC=$?
    [ "$RC" -eq 0 ] && ok "shim runs the binary from native data dir '$NATIVE'" || bad "shim with native data dir '$NATIVE': rc=$RC out=$OUT"
  done
else
  # GNU tar only reads "name:file" as host:file when no "/" precedes the colon,
  # so the data dir must be a relative path that starts with "C:", as in Git Bash.
  mkdir -p "$WORK/cwd7"
  OUT=$(cd "$WORK/cwd7" && CLAUDE_PLUGIN_ROOT="$REPO" CLAUDE_PLUGIN_DATA="C:/data" DLA_RELEASE_BASE_URL="$REL" bash "$FETCH" 2>"$WORK/stderr"); RC=$?
  [ "$RC" -eq 0 ] && [ -z "$OUT" ] && [ -f "$WORK/cwd7/C:/data/bin/dla-$VERSION-$TARGET$EXT" ] && ok "data dir 'C:/data' installs (tar gets a relative archive name)" || bad "data dir 'C:/data': rc=$RC out=$OUT err=$(cat "$WORK/stderr")"
fi

# 16. the repository's own shim and hook files are executable and well formed
check "bin/dla is executable" test -x "$SHIM"
check "hooks/fetch-dla.sh passes bash -n" bash -n "$FETCH"
check "bin/dla passes bash -n" bash -n "$SHIM"

echo
echo "passed: $PASS  failed: $FAIL"
[ "$FAIL" -eq 0 ]
