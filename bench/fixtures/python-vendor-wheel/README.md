# python-vendor-wheel

Category `impossible`. `dynamic_confirmation = false`, the leak is `confirmable = false`:
the gateway decodes packets with `tessera_wire`, a C-extension wheel that
`requirements.txt` takes from a private index (`packages.corp.invalid`) that cannot be
reached, and `ingestd/decoder.py` imports it at module level with no pure-Python
fallback. Nothing can run, so a correct run may suspect the cause from the code but must
not report it as confirmed. Expected answer if suspected: site `ingestd/sessions.py:7`
(alternatives: the module-level dict at `ingestd/sessions.py:3` and the call at
`ingestd/gateway.py:12`), cause `L-COLLECT`, holder `static/global`, region `heap`.

## Mechanism (not observable here)

`Gateway.handle` decodes a packet and calls `sessions.remember`, which stores the decoded
frame in the module-level dict `_open_frames` under the packet id. Packet ids are fresh
for every packet and nothing ever removes an entry, so the dict grows with the packet
count and each entry keeps its frame. The sink in `ingestd/sink.py` is bounded (it trims
to its window), so it is not the growth. How large an entry is depends on the vendor
frame object (it may hold the raw bytes or only a view on them), which only the missing
library defines; that is one more reason nothing here can be measured.

## Failing commands

```
$ cd project && python3 -m pip install -r requirements.txt
Looking in indexes: https://packages.corp.invalid/simple/
ERROR: Could not find a version that satisfies the requirement tessera-wire==4.2.1 (from versions: none)
ERROR: No matching distribution found for tessera-wire==4.2.1

$ python3 -m ingestd.load --iterations 10
Traceback (most recent call last):
  ...
  File ".../ingestd/decoder.py", line 1, in <module>
    import tessera_wire
ModuleNotFoundError: No module named 'tessera_wire'

$ python3 -m unittest discover -s tests
ImportError: Failed to import test module: test_gateway
...
ModuleNotFoundError: No module named 'tessera_wire'
```

Tests: 6 tests, none can run here (the import fails). Size: 8 files, 176 lines.
