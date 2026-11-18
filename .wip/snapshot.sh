#!/bin/bash
# Snapshot the working tree of /home/user/dla onto branch wip/progress and push it.
# Keeps the branch's .wip/ notes; files in the scratchpad's wip-notes/ replace notes of the same name.
# Never touches main, the real index or the working tree.
set -euo pipefail
cd /home/user/dla
label="${1:-snapshot}"
exec 9>/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u7/snapshot.lock
flock 9
export GIT_INDEX_FILE=/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u7/snapshot.index
rm -f /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u7/snapshot.index
git read-tree HEAD
git add -A
parent=$(git rev-parse refs/heads/wip/progress)
git read-tree --prefix=.wip/ "$parent:.wip"
for f in /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/wip-notes/*; do
  [ -f "$f" ] || continue
  blob=$(git hash-object -w "$f")
  git update-index --add --cacheinfo "100644,$blob,.wip/$(basename "$f")"
done
tree=$(git write-tree)
if [ "$tree" = "$(git rev-parse "$parent^{tree}")" ]; then echo "no change since $parent"; exit 0; fi
commit=$(GIT_AUTHOR_NAME="Mykyta Hriaznov" GIT_AUTHOR_EMAIL="17167893+ngriaznov@users.noreply.github.com" \
         GIT_COMMITTER_NAME="Mykyta Hriaznov" GIT_COMMITTER_EMAIL="17167893+ngriaznov@users.noreply.github.com" \
         git commit-tree "$tree" -p "$parent" -m "Snapshot: $label")
git update-ref refs/heads/wip/progress "$commit" "$parent"
for i in 1 2 3 4; do git push -q origin wip/progress && break; sleep $((2**i)); done
echo "snapshot $commit ($label)"
