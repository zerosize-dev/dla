#!/usr/bin/env bash
# SessionStart hook: make sure the dla binary for this platform and plugin version
# is installed under ${CLAUDE_PLUGIN_DATA}/bin. Runs once per plugin version.
#
# Contract:
#   - success or nothing to do: prints nothing, exits 0
#   - any failure: prints one short message naming the exact path where the
#     binary can be placed by hand, exits 0 so the session continues
#   - progress and diagnostics go to stderr
#   - writes only inside ${CLAUDE_PLUGIN_DATA}; never into the project or the plugin root
#
# Environment:
#   CLAUDE_PLUGIN_DATA        data directory (set by Claude Code)
#   CLAUDE_PLUGIN_ROOT        plugin root (set by Claude Code; derived from this file if absent)
#   DLA_RELEASE_BASE_URL      override of the release download base: an https URL,
#                             a file:// URL or a plain directory path
#   DLA_TARGET                override of target detection (tests, cross installs)
#
# Portable to bash 3.2 (macOS), Linux and Git Bash on Windows: no arrays, no
# associative arrays, no ${var,,}, no jq.

RELEASE_REPO_URL="https://github.com/zerosize-dev/dla/releases/download"
TMP_DIR=""

log() { printf 'dla: %s\n' "$*" >&2; }

cleanup() {
  if [ -n "$TMP_DIR" ] && [ -d "$TMP_DIR" ]; then
    rm -rf "$TMP_DIR"
  fi
}
trap cleanup EXIT
trap 'exit 0' HUP INT TERM

# Print the manual-install instructions on stdout (Claude Code adds hook stdout
# to the session context) and leave with status 0.
fail() {
  log "$1"
  if [ -n "${DEST:-}" ]; then
    printf 'DLA binary is not installed (%s). Place the dla executable for %s at: %s (then chmod +x it). Release assets: %s\n' \
      "$1" "${TARGET:-this platform}" "$DEST" "${BASE_URL:-$RELEASE_REPO_URL}"
  else
    printf 'DLA binary is not installed (%s).\n' "$1"
  fi
  exit 0
}

detect_target() {
  if [ -n "${DLA_TARGET:-}" ]; then
    printf '%s' "$DLA_TARGET"
    return 0
  fi
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

plugin_version() {
  # First top-level "version" key; plugin.json is small and hand-maintained.
  sed -n 's/^[[:space:]]*"version"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$1" 2>/dev/null | head -n 1
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" 2>/dev/null | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" 2>/dev/null | awk '{print $1}'
  else
    return 1
  fi
}

# fetch <url-or-path> <output file>; returns non-zero on failure.
fetch() {
  src=$1
  out=$2
  case "$src" in
    file://*)
      path=${src#file://}
      # file:///C:/x on Windows leaves a leading slash before the drive letter.
      case "$path" in /[A-Za-z]:/*) path=${path#/} ;; esac
      cp "$path" "$out" 2>/dev/null
      ;;
    http://*|https://*)
      if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 2 --connect-timeout 15 --max-time 90 -o "$out" "$src" 2>/dev/null
      elif command -v wget >/dev/null 2>&1; then
        wget -q --tries=2 --timeout=30 -O "$out" "$src" 2>/dev/null
      else
        log "neither curl nor wget is available"
        return 1
      fi
      ;;
    *)
      cp "$src" "$out" 2>/dev/null
      ;;
  esac
}

main() {
  script_dir=$(cd "$(dirname "$0")" 2>/dev/null && pwd)
  root=${CLAUDE_PLUGIN_ROOT:-"$script_dir/.."}
  manifest="$root/.claude-plugin/plugin.json"

  VERSION=$(plugin_version "$manifest")
  if [ -z "$VERSION" ]; then
    fail "cannot read the plugin version from $manifest"
  fi

  TARGET=$(detect_target) || fail "unsupported platform $(uname -s 2>/dev/null)/$(uname -m 2>/dev/null)"
  EXT=""
  case "$TARGET" in *windows*) EXT=".exe" ;; esac

  if [ -z "${CLAUDE_PLUGIN_DATA:-}" ]; then
    fail "CLAUDE_PLUGIN_DATA is not set; run this hook from Claude Code"
  fi
  BIN_DIR="$CLAUDE_PLUGIN_DATA/bin"
  DEST="$BIN_DIR/dla-$VERSION-$TARGET$EXT"
  BASE_URL=${DLA_RELEASE_BASE_URL:-"$RELEASE_REPO_URL/v$VERSION"}
  BASE_URL=${BASE_URL%/}

  # Once per plugin version: nothing to do when the binary is already there.
  if [ -f "$DEST" ]; then
    exit 0
  fi

  mkdir -p "$BIN_DIR" 2>/dev/null || fail "cannot create $BIN_DIR"
  TMP_DIR=$(mktemp -d "$BIN_DIR/.fetch.XXXXXX" 2>/dev/null) || fail "cannot create a temporary directory in $BIN_DIR"

  asset="dla-$TARGET.tar.gz"
  log "downloading $asset for dla $VERSION"
  fetch "$BASE_URL/$asset" "$TMP_DIR/$asset" || fail "download of $BASE_URL/$asset failed"
  fetch "$BASE_URL/$asset.sha256" "$TMP_DIR/$asset.sha256" || fail "download of $BASE_URL/$asset.sha256 failed"

  expected=$(awk 'NR == 1 {print $1}' "$TMP_DIR/$asset.sha256" | tr 'A-F' 'a-f')
  case "$expected" in
    *[!0-9a-f]*|"") fail "$asset.sha256 does not start with a SHA-256 digest" ;;
  esac
  if [ "${#expected}" -ne 64 ]; then
    fail "$asset.sha256 does not start with a SHA-256 digest"
  fi

  actual=$(sha256_of "$TMP_DIR/$asset") || fail "neither sha256sum nor shasum is available to verify the download"
  if [ "$actual" != "$expected" ]; then
    fail "SHA-256 mismatch for $asset (expected $expected, got $actual); nothing was installed"
  fi

  mkdir "$TMP_DIR/x" 2>/dev/null || fail "cannot create a temporary directory in $BIN_DIR"
  tar -xzf "$TMP_DIR/$asset" -C "$TMP_DIR/x" 2>/dev/null || fail "cannot extract $asset"
  if [ ! -f "$TMP_DIR/x/dla$EXT" ]; then
    fail "$asset does not contain dla$EXT at its root"
  fi

  chmod +x "$TMP_DIR/x/dla$EXT" 2>/dev/null
  # Same directory tree, so this rename is atomic on every platform we target.
  mv -f "$TMP_DIR/x/dla$EXT" "$DEST" 2>/dev/null || fail "cannot move the binary to $DEST"
  log "installed $DEST"
  exit 0
}

main "$@"
