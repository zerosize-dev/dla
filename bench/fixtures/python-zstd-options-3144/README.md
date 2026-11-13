# python-zstd-options-3144

Category: `archetype-free` (the trap half of a version pair). Pair: `python-zstd-options-3145`.
Cause `UNCLASSIFIED`, holder `native-handle/global-ref`, region `heap`.
Toolchain: `[toolchain] python = "3.14.4"`; `project/.python-version` says `3.14.4`.

## Mechanism (for authors)

The two fixtures have the same code, the same prompt and the same expected site. They
differ in `.python-version` and in the Python release the runner provides: 3.14.4 here,
3.14.5 in the decoy.

`SegmentReader._decoder` builds `zstd.ZstdDecompressor(options=...)` for the tenant. The
options come from the tenant's stored override document (JSON, parsed on every read) through
`options.decoder_options`, which maps the names it knows to zstd parameter ids and passes
every other name through unchanged. A tiered tenant's document has a `window_ladder` key
with a list of 64 sizes. That key is a string, not a parameter id, so the decoder
constructor raises `TypeError`, and `_decoder` falls back to a default decoder (two thirds
of the tenants). That fallback is correct behaviour in every release.

In CPython 3.14.4 the C code of `compression.zstd` takes a new reference to the value of a
rejected option and does not release it on the error path (the same happens for an
integer key that is out of range). Each failed construction strands the value, here the
parsed ladder: a list of 64 integers, about 2.6 KB and 65 allocated blocks. Nothing in the
Python program refers to it, so no container, field or closure holds it; the reference is
owned by the extension module. Release 3.14.5 releases it (the decoy). Retention is
reported by the process, not by any holder `dla census` can name, which is why no taxonomy row fits and
the expected label is `UNCLASSIFIED`.

`archive/fallbacks.py` keeps the last fallback reason per tenant in a module-level dict. It has
one key per tenant (30), so it is a bounded look-alike that the code sweep and the census
report and that the harness shows flat.

A different edit would not remove it: bounding the override documents, caching the parsed
dict or dropping the unknown names removes the failing construction, but the cause
(the extension's missing release) is the same; the version is the discriminator.

## Measurements

Harness outside `project/` (drives the reader as `archive.load.run` does, `gc.collect()`
before each reading, `tracemalloc` started after the imports; `blocks` is
`sys.getallocatedblocks()` over the baseline; the fallback count is the reader's own),
with the interpreter the fixture names, installed with `uv python install 3.14.4`:

```
python3 -B harness.py <project dir> 2000
n=2000 i=500 traced_kb=773 blocks=22056 fallbacks=334
n=2000 i=1000 traced_kb=1543 blocks=44034 fallbacks=667
n=2000 i=1500 traced_kb=2313 blocks=66012 fallbacks=1000
n=2000 i=2000 traced_kb=3085 blocks=88056 fallbacks=1334
python3 -B harness.py <project dir> 8000
n=8000 i=2000 traced_kb=3085 blocks=88056 fallbacks=1334
n=8000 i=4000 traced_kb=6168 blocks=176034 fallbacks=2667
n=8000 i=6000 traced_kb=9250 blocks=264012 fallbacks=4000
n=8000 i=8000 traced_kb=12335 blocks=352056 fallbacks=5334
```

Growth is linear in reads: 1.54 KB per read (3085 KB at 2k, 12335 KB at 8k, slope 1.54
KB/read) and 44 blocks per read, which is 65 blocks per fallback read. `ru_maxrss` was 31 MB
after 2k and 68 MB after 8k. On 3.14.5 the same harness reads 0 KB and 11 to 12 blocks at every
point (see `python-zstd-options-3145`).

Workload: `python3 -m archive.load --iterations 8000` takes about 0.6 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (9 tests, pass on 3.14.4 and
3.14.5; they need 3.14 for `compression.zstd`).
Size: 8 source files, 209 lines.
