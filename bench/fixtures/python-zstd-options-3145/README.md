# python-zstd-options-3145

Category: `decoy-control` (the control half of a version pair). Pair: `python-zstd-options-3144`.
Look-alike cause: `N-WARM`. Toolchain: `[toolchain] python = "3.14.5"`;
`project/.python-version` says `3.14.5`.

## Mechanism (for authors)

Same code, prompt and site as `python-zstd-options-3144`; the only difference is the Python
release (3.14.5) and the matching `.python-version`. Two thirds of the reads have an
override document with a `window_ladder` key (a string, not a parameter id), so the
decoder constructor rejects the options and `SegmentReader._decoder` falls back to a
default decoder. That looks like the same fault as in the trap, but on 3.14.5 the
extension releases the rejected value, and nothing is retained: memory is flat from the
first reading. The fallback counter rises with the reads, the memory does not.
`archive/fallbacks.py` keeps the last fallback reason per tenant in a module-level dict with
one key per tenant (30): a bounded look-alike that the code sweep and the census report.

## Measurements

Same harness and counts as the trap, with the interpreter installed by
`uv python install 3.14.5`:

```
python3 -B harness.py <project dir> 2000
n=2000 i=500 traced_kb=0 blocks=11 fallbacks=334
n=2000 i=1000 traced_kb=0 blocks=11 fallbacks=667
n=2000 i=1500 traced_kb=0 blocks=11 fallbacks=1000
n=2000 i=2000 traced_kb=0 blocks=11 fallbacks=1334
python3 -B harness.py <project dir> 8000
n=8000 i=2000 traced_kb=0 blocks=12 fallbacks=1334
n=8000 i=4000 traced_kb=0 blocks=12 fallbacks=2667
n=8000 i=6000 traced_kb=0 blocks=12 fallbacks=4000
n=8000 i=8000 traced_kb=0 blocks=12 fallbacks=5334
```

Traced memory is 0 KB over the baseline and the block count stays at base + 11 or 12 at 2000
and 8000 reads, while 5334 of 8000 reads take the fallback. `ru_maxrss` was 23 MB at both counts.
On 3.14.4 the same workload reads 3085 KB at 2k and 12335 KB at 8k.

Workload: `python3 -m archive.load --iterations 8000` takes about 0.6 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (9 tests, pass).
Size: 8 source files, 209 lines.
