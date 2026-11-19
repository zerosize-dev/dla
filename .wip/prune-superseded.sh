#!/bin/bash
# Frees disk in /home/user/dla/target/debug/build (cargo's per-package layout:
# build/<package>/<hash>/{fingerprint,out}). Units of one package that produce the same set of
# output names (hash stripped) are versions of one build unit: keep the 2 newest of each set and
# anything written in the last hour; a removed unit is rebuilt when needed.
python3 - <<'PY'
import os, re, time, shutil, collections
root = '/home/user/dla/target/debug/build'
now = time.time(); freed = 0; removed = 0
if not os.path.isdir(root):
    print('nothing to prune'); raise SystemExit(0)
for pkg in os.listdir(root):
    pdir = os.path.join(root, pkg)
    if not os.path.isdir(pdir) or not pkg.startswith('dla'):
        continue
    groups = collections.defaultdict(list)
    for h in os.listdir(pdir):
        d = os.path.join(pdir, h); out = os.path.join(d, 'out')
        if not re.fullmatch(r'[0-9a-f]{16}', h) or not os.path.isdir(out):
            continue
        names = tuple(sorted(re.sub(r'-[0-9a-f]{16}', '', f) for f in os.listdir(out)))
        if not names:
            continue
        mt = max(os.stat(os.path.join(out, f)).st_mtime for f in os.listdir(out))
        groups[names].append((mt, d))
    for v in groups.values():
        v.sort(reverse=True)
        for mt, d in v[2:]:
            if now - mt > 3600:
                size = sum(os.path.getsize(os.path.join(r, f)) for r, _, fs in os.walk(d) for f in fs)
                shutil.rmtree(d); freed += size; removed += 1
print(f'pruned {removed} units, {freed // (1 << 20)} MB')
PY
