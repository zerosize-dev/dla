#!/bin/bash
# Delete test executables in target/debug/deps that have the same name as a newer one
# (older hash). Never touches .rlib/.rmeta/.so/.d/.o or hard-linked files.
cd /home/user/dla/target/debug/deps || exit 1
find . -maxdepth 1 -type f -links 1 -perm -u+x -regextype posix-extended -regex '\./[A-Za-z0-9_]+-[0-9a-f]{16}' -printf '%T@ %s %f\n' \
 | awk '{n=$3; sub(/-[0-9a-f]{16}$/,"",n); print n, $1, $2, $3}' | sort -k1,1 -k2,2nr \
 | awk '{if($1==prev){print $4} prev=$1}' | xargs -r rm -f -v -- | wc -l
# Old copies of the workspace crates' libraries (libdla_*.rlib/.rmeta): keep the 3 newest per
# name and anything written in the last hour; a removed copy is rebuilt when needed.
python3 - <<'PY' >/dev/null
import os,re,time,collections
now=time.time(); g=collections.defaultdict(list)
for f in os.listdir('.'):
    m=re.match(r'(libdla_[a-z_]+)-([0-9a-f]{16})\.(rlib|rmeta)$', f)
    if m and os.path.isfile(f) and os.stat(f).st_nlink==1:
        g[(m.group(1),m.group(3))].append((os.stat(f).st_mtime,f))
for v in g.values():
    v.sort(reverse=True)
    for mt,f in v[3:]:
        if now-mt>3600: os.remove(f)
PY
