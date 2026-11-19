#!/bin/bash
# Usage: ci-wait.sh <commit sha> <out dir>
# Waits up to 8 minutes for the CI run of <sha> on wip/progress. Prints "PENDING ..." if it has
# not finished (call again), or the job conclusions; failed job logs go to <out dir>/<job>.log.
set -u
sha="$1"; out="$2"; br="${3:-wip/progress}"; max="${4:-480}"; mkdir -p "$out"
end=$((SECONDS+max))
while :; do
  line=$(gh run list --repo zerosize-dev/dla --workflow ci.yml --branch "$br" --limit 10 \
        --json databaseId,headSha,status,conclusion \
        --jq ".[] | select(.headSha==\"$sha\") | \"\(.databaseId) \(.status) \(.conclusion)\"" 2>/dev/null | head -1)
  id=${line%% *}; st=$(echo "$line" | cut -d' ' -f2)
  if [ -n "$id" ] && [ "$st" = "completed" ]; then break; fi
  if [ $SECONDS -ge $end ]; then echo "PENDING run=${id:-none} status=${st:-unknown}"; exit 0; fi
  sleep 30
done
echo "DONE run=$id conclusion=$(echo "$line" | cut -d' ' -f3)"
gh run view "$id" --repo zerosize-dev/dla --json jobs --jq '.jobs[] | "\(.conclusion)\t\(.name)"'
gh run view "$id" --repo zerosize-dev/dla --log-failed > "$out/failed.raw" 2>/dev/null || true
cut -f1 "$out/failed.raw" | sort -u | while IFS= read -r job; do
  [ -n "$job" ] || continue
  f="$out/$(echo "$job" | tr -c 'A-Za-z0-9\n' '_').log"
  grep -F "$job" "$out/failed.raw" | cut -f3- | cut -c30- | sed 's/\x1b\[[0-9;]*m//g' > "$f"
  echo "log: $f ($(wc -l < "$f") lines)"
done
