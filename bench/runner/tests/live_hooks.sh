#!/usr/bin/env bash
# Live check of the dla gate hooks with a real Claude Code session (`claude -p`).
#
# Opt-in: nothing runs unless DLA_LIVE_HOOKS=1. Each step is one short headless run on the
# haiku model with --max-turns 8 (about one cent each). The plugin under test is this
# repository (--plugin-dir), so the hooks are the ones in hooks/hooks.json; a scratch project
# holds an active run made by `dla run init`.
#
# Steps:
#   1. Write of the run's report.md is refused by the write gate.
#   2. Read of a *-archetypes.md file (a canary) is refused before the classify gate.
#   3. The same Read succeeds after `dla gate check --stage classify` passed.
#   4. The dla:worker subagent's SubagentStop reaches `dla gate stop-hook --subagent`.
#
# The result of each step is read from the stream-json hook events (--include-hook-events)
# and from the files on disk, not from what the model says. Exit code 0 when every step
# behaved as designed, 1 when one did not, 2 when the check cannot run.
#
# Environment: DLA_BIN (default target/debug/dla in this repository), DLA_LIVE_KEEP=1 to keep
# the scratch directory and print its path.

set -u

if [ "${DLA_LIVE_HOOKS:-}" != 1 ]; then
  echo "live_hooks.sh: skipped (set DLA_LIVE_HOOKS=1 to run it; it uses the claude CLI)"
  exit 0
fi

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../../.." && pwd)
DLA_BIN=${DLA_BIN:-$repo/target/debug/dla}

if ! command -v claude >/dev/null 2>&1; then
  echo "live_hooks.sh: the claude CLI is not on PATH" >&2
  exit 2
fi
if [ ! -x "$DLA_BIN" ]; then
  echo "live_hooks.sh: no dla binary at $DLA_BIN (cargo build -p dla-cli, or set DLA_BIN)" >&2
  exit 2
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "live_hooks.sh: python3 is needed to read the stream-json output" >&2
  exit 2
fi

scratch=$(mktemp -d)
cleanup() {
  if [ "${DLA_LIVE_KEEP:-}" = 1 ]; then
    echo "scratch kept at $scratch"
  else
    rm -rf "${scratch:?}"
  fi
}
trap cleanup EXIT

proj=$scratch/proj
mkdir -p "$proj/references"
printf 'CANARY-LIVE-7f3a: the archetype table says L-COLLECT.\n' >"$proj/references/go-archetypes.md"

# The scratch project, with an active symptom run and the consent of the runner.
export DLA_BIN
unset DLA_RUN DLA_WORKER CLAUDE_PROJECT_DIR
init=$(DLA_CONSENT_TIERS=1 "$DLA_BIN" --project "$proj" run init --entry symptom --session-id live)
run_id=$(printf '%s' "$init" | python3 -c 'import json,sys; print(json.load(sys.stdin)["run_id"])') || {
  echo "live_hooks.sh: dla run init failed: $init" >&2
  exit 2
}
report=$proj/.dla/$run_id/report.md

fails=0
note() { printf '%s\n' "$*"; }
fail() { printf 'FAIL: %s\n' "$*"; fails=$((fails + 1)); }
pass() { printf 'ok:   %s\n' "$*"; }

# claude_run <out file> <prompt>: one headless run in the project.
claude_run() {
  out=$1
  shift
  (
    cd "$proj" || exit 2
    claude -p "$1" \
      --plugin-dir "$repo" \
      --model haiku \
      --max-turns 8 \
      --allowedTools "Write Read Agent" \
      --output-format stream-json --verbose --include-hook-events \
      --no-session-persistence
  ) >"$out" 2>"$out.err"
}

# summary <out file>: one line per hook response and per tool result, for the log and checks.
summary() {
  python3 - "$1" <<'PY'
import json, sys
for line in open(sys.argv[1]):
    try:
        e = json.loads(line)
    except ValueError:
        continue
    t = e.get("type")
    if t == "system" and str(e.get("subtype", "")).startswith("hook"):
        print("HOOK", e.get("subtype"), e.get("hook_event"), e.get("hook_name"),
              e.get("outcome") or e.get("exit_code"), (e.get("stderr") or e.get("stdout") or "")[:200].replace("\n", " "))
    elif t == "user":
        for c in (e.get("message", {}).get("content") or []):
            if isinstance(c, dict) and c.get("type") == "tool_result":
                body = c.get("content")
                if isinstance(body, list):
                    body = " ".join(x.get("text", "") for x in body if isinstance(x, dict))
                print("RESULT", "error" if c.get("is_error") else "ok", str(body)[:200].replace("\n", " "))
    elif t == "result":
        print("DONE", e.get("subtype"), "cost", e.get("total_cost_usd"))
PY
}

# 1. Write of report.md is refused.
note "step 1: Write of report.md"
claude_run "$scratch/s1.jsonl" "Use the Write tool to write the file $report with the content x. If the tool is blocked, say BLOCKED and stop. Do nothing else."
summary "$scratch/s1.jsonl" | sed 's/^/      /'
if [ -e "$report" ]; then
  fail "report.md was written despite the write gate"
else
  if summary "$scratch/s1.jsonl" | grep -q '^HOOK .*PreToolUse'; then
    pass "report.md not written, and a PreToolUse hook ran"
  else
    fail "report.md not written, but no PreToolUse hook event was seen (the model may not have tried)"
  fi
fi

# 2. Read of the canary before classify is refused: the canary never reaches the model.
note "step 2: Read of go-archetypes.md before the classify gate"
claude_run "$scratch/s2.jsonl" "Use the Read tool on $proj/references/go-archetypes.md and reply with its first line, copied exactly. If the tool is blocked, say BLOCKED and stop."
summary "$scratch/s2.jsonl" | sed 's/^/      /'
if grep -q 'CANARY-LIVE-7f3a' "$scratch/s2.jsonl"; then
  fail "the canary text reached the session before the classify gate"
else
  pass "the canary text did not reach the session"
fi

# 3. After the classify gate has passed, the same Read goes through.
note "step 3: Read of go-archetypes.md after the classify gate"
for item in intake.target intake.scope intake.symptom intake.runtimes intake.build intake.workload intake.consent intake.budget; do
  "$DLA_BIN" --project "$proj" ledger resolve "$item" --status answered --evidence E1 >/dev/null
done
gate=$("$DLA_BIN" --project "$proj" gate check --stage classify)
if printf '%s' "$gate" | python3 -c 'import json,sys; sys.exit(0 if json.load(sys.stdin).get("ok") else 1)'; then
  claude_run "$scratch/s3.jsonl" "Use the Read tool on $proj/references/go-archetypes.md and reply with its first line, copied exactly."
  summary "$scratch/s3.jsonl" | sed 's/^/      /'
  if grep -q 'CANARY-LIVE-7f3a' "$scratch/s3.jsonl"; then
    pass "the canary was readable after the classify gate"
  else
    fail "the canary was not readable after the classify gate"
  fi
else
  fail "gate check --stage classify did not pass: $gate"
fi

# 4. The worker subagent's SubagentStop hook (from hooks/hooks.json) fires.
note "step 4: dla:worker SubagentStop"
claude_run "$scratch/s4.jsonl" "Use the Agent tool once with subagent_type dla:worker and the prompt 'Reply with the word done and nothing else. Do not run any command.' Then reply with the word finished."
summary "$scratch/s4.jsonl" | sed 's/^/      /'
if summary "$scratch/s4.jsonl" | grep -q '^HOOK .*SubagentStop'; then
  pass "a SubagentStop hook ran for the dla:worker subagent"
else
  fail "no SubagentStop hook event was seen (the agent may not have started)"
fi

echo
if [ "$fails" -eq 0 ]; then
  echo "live_hooks.sh: all steps behaved as designed"
  exit 0
fi
echo "live_hooks.sh: $fails step(s) did not behave as designed"
exit 1
