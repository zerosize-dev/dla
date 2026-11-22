# Real plugin run pillow-1 (Pillow 10.1.0), 2026-11-22 12:47-12:51 UTC

Build: main at 29cb6f6 (`dla 0.9.0-preview.4`). Model claude-sonnet-5-5 through
/root/dla-real/run.py. The run ended by itself: exit 0, 219 s, 40 turns, about $0.65.
Evidence: <sp>/u36/pillow-1 (run.json, transcript.jsonl, dla/).

## Ground truth (by hand, <sp>/u72)

`ImageDraw.text` -> `ImageFont.getmask2` -> C `font_render` (src/_imagingft.c) leaks one int per
call: `PyObject_GetAttrString(image, "id")` is never decremented. tracemalloc sees it at
`src/PIL/ImageFont.py:580` (`self.font.render(`), 28 B and 1 object per call. No holder: no census
candidate names it. Two of the project's test files (test_imagedraw.py, test_imagefont.py) run in
a loop in one process with pytest.main: the top allocation site that grows every round is
`src/PIL/ImageFont.py:580` (about 145 objects per round); next is pytest's own (14). 17 s.

## Result

Report status COMPLETE, 3 of 180 budget minutes used. Leak not found, text drawing never called.
"confirmed 0, confirmed but disputed by the verifier 3" = the plugin registries Image.ID, OPEN,
EXTENSION, grown by a driver that registers a new format per unit (a fair N-INTENT).
Queue: 17 of 346 ranked candidates targeted in 3 harnesses; 329 not targeted.

## Causes

1. One blocker closes every queue. The model generated a native harness (H3, a C loop driver),
   saw that the C sources are CPython extension modules, and recorded
   `dla ledger resolve intake.workload --status unresolved --reason build_fails --remedy "native
   candidates in src/*.c need a CPython-embedding C driver ..."`. The tool's own text offers that
   ("that closes the queue of the runtime"). `intake.workload` is one item per run: status then
   showed BOTH queues `closed: blocked` (python had 183 pending and two harnesses that ran), and
   `dla ledger close-rest` closed 567 findings (563 suspected_static_only / not_exercised).
2. A C driver is offered for the C sources of a Python extension. They can only be driven through
   Python.
3. No workload-first pass for Python: a leak without a holder shows only as a growing allocation
   site under a broad workload. The project's own tests in a loop show it at once (above); the
   Python queue has no test loop (JVM only).
4. Residual open items (O1..O4: heap growth above threshold in S1..S4, os.rss steady 96.6 B per
   unit in S2) were resolved `inconclusive` with a remedy text, no allocation capture, with 177
   minutes of budget left. The item's description says "attribute it first: dla harness run H1
   --mode fresh --capture alloc,histogram"; its `fix` is the bare resolve command.
5. The model had to build the C extension itself (copy to $TMPDIR, patch setup.py for Python 3.13,
   copy the .so files into src/PIL). It managed. The driver's units were image open/save/resize/
   filter/convert in six formats; no text.
