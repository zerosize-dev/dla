#!/bin/bash
# Usage: commit-snapshot-tree.sh <snapshot sha on wip/progress> <message file>
# Commits the tree of a snapshot (without .wip/) on top of main, without touching the working
# tree: other work in the working tree stays as uncommitted changes. Prints the new commit.
set -euo pipefail
snap="$1"; msg="$2"
cd /home/user/dla
[ "$(git symbolic-ref --short HEAD)" = main ] || { echo "HEAD is not main"; exit 1; }
idx=$(mktemp -u /tmp/claude-0/idx.XXXXXX)
GIT_INDEX_FILE=$idx git read-tree "$snap"
GIT_INDEX_FILE=$idx git rm -r -q --cached .wip
tree=$(GIT_INDEX_FILE=$idx git write-tree)
rm -f "$idx"
who="Mykyta Hriaznov"; mail="17167893+ngriaznov@users.noreply.github.com"
c=$(GIT_AUTHOR_NAME="$who" GIT_AUTHOR_EMAIL="$mail" GIT_COMMITTER_NAME="$who" GIT_COMMITTER_EMAIL="$mail" \
    git commit-tree "$tree" -p main -F "$msg")
git update-ref refs/heads/main "$c"
git reset -q
echo "$c"
git log -1 --format='%h %an <%ae> %s' "$c"
git diff --stat HEAD~1 HEAD | tail -1
