# Brief R1: the Python agent says who holds the objects of a growing line

Repository: /home/user/dla. You change ONE file of the product, `harness/python/sitecustomize.py`
(the Python agent), plus `harness/python/README.md` where it lists the captures. Scratch directory:
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u91 (your test
scripts and outputs go there). NO cargo in this brief: `target/` is empty and the disk is needed
by another job. You test with Python only. The Rust side (the capture flag, the summary, the
tests in the Rust suite) is a later brief.

## Why

In a Python driver run the agent counts, at every checkpoint, the live traced blocks per project
line (`py.blocks`, see the comment block "py.blocks: live traced blocks per allocation site" and
`_blocks_checkpoint`). A line that grows says WHERE the surviving objects were made. It does not
say WHO keeps them, and that is the diagnosis.

A real run on a Python library: twelve lines grew in the loop over the project's tests. One was a
leak in a C extension (nothing in Python holds those objects). The other eleven were objects of
ordinary tests, all kept by one holder that no line pointed at: a module-level object that stores
an exception and re-raises that same exception object on every attribute access
(`raise self.ex`). Each raise adds the raising frames to the exception's `__traceback__`, and the
frames keep their locals, and through them every object of the caller. Walking
`gc.get_referrers` by hand from one surviving object named the holder in one step:

    object <- dict <- test instance <- ... <- frame <- traceback <- traceback <- ImportError
           <- <holder class>.ex <- module `pkg.mod` global `_name`

The agent can do that walk itself.

## What to build

A new capture kind for the Python agent, `heap`, taken like the other heavy captures (read
`_capture_now`, `_dump_here`, `_capture_tracemalloc`, `_capture_types` and the comment block
"raw captures (DLA_CAPTURE)" first; follow their conventions for files, the `captures` array of
the record, limitations and never raising).

1. Which lines. With `heap` in `DLA_CAPTURE`, at the first dump point remember the block count
   per site (the same site rule and the same counting as `py.blocks`; reuse the functions, do not
   copy them). At the LAST dump point (the final one of the process) compute the growth per site
   and take the sites that grew, largest growth first, at most 12. If there was only one dump
   point, take the largest sites and say so in a limitation.
2. Which objects. For those sites find the live objects allocated there:
   `tracemalloc.get_object_traceback(o)` resolved with the same site rule. Look at every object
   the collector tracks (`gc.get_objects()`) and at every object a tracked object refers to
   (`gc.get_referents`), so that untracked objects (strings, numbers, bytes, tuples of them,
   objects of extension types) that some container holds are found too. Per site keep at most 40
   objects, spread over the whole scan and not the first 40 (reservoir or stride), and count how
   many were seen in total.
3. Who holds them. Build the referrer relation ONCE (one pass of `gc.get_referents` over the
   tracked objects: child id -> parents), not `gc.get_referrers` per object. From each kept
   object search breadth first toward a root, at most 40 steps deep, skipping the agent's own
   objects (its lists, dicts and frames of this walk: mark them, and exclude the frame of the
   running capture). A root is, in this order of preference when several are reachable at the
   same depth: (a) the `__dict__` of a module that is in `sys.modules` (root kind `module-global`,
   with the module name and the key); (b) a class (`class-attribute`, qualified class name and
   attribute); (c) a frame that is running or suspended on a live thread, generator or
   coroutine (`frame`, function name and file:line); (d) an object nothing tracked refers to
   (`no-referrer`: held from native code or by the interpreter). Prefer the shortest chain.
4. Each step of a chain says: the type of the object (`module.QualName`), how the next object
   is reached from it when that can be read cheaply (dict key as a short repr, attribute name
   found through the instance `__dict__`, list or tuple index, `__traceback__`, `tb_frame`,
   `tb_next`, `f_locals` name, cell of a closure, `__self__`), and a project-relative
   `file:line` when the object has one (a function or a frame: its code's file and first line or
   current line; a traceback: `tb_lineno` in its frame's file; an instance of a class defined in
   the project: nothing, the type name is enough). Strings are cut to 80 characters. Values are
   never written unless the run allows raw values (find how the agent decides that for its other
   captures and follow it; a key of a dict counts as a value).
5. Group the chains of a site by their signature (the types, the links and the root, without
   indexes of consecutive `tb_next` links: a run of tracebacks counts as one step with a count),
   and write for each group how many of the kept objects it holds. Also write per site: blocks at
   the first and last dump, objects found (tracked and untracked separately), objects kept, and
   when blocks grew and NO object was found for the site, the sentence that the blocks do not
   belong to an object any Python container refers to, so native code holds them or a reference
   was lost there.
6. File: `py-<n>.retain.json` next to the other capture files, schema
   `{"dla_pyretain": 1, "frames": N, "sites": [ {"site", "blocks_first", "blocks_last",
   "found_tracked", "found_untracked", "kept", "note"?, "chains": [ {"count", "root": {"kind",
   ...}, "steps": [ {"type", "via"?, "at"?, "repeat"?} ] } ] } ], "limitations": [...]}`.
   Add its entry to the record's `captures` array the way the alloc capture does (kind `heap`).
   Update the line in `_capture_now` that says which kinds the python agent writes.
7. Bounds, all named constants at the top of the section: 12 sites, 40 kept objects per site, 40
   steps, and a time bound for the whole capture (default 60 s, checked between objects; what was
   not done is said in `limitations`). The capture must free everything it built before it
   returns (the reverse relation can be large) and must not leave a reference to any project
   object in the agent's globals.
8. `harness/python/README.md`: add the capture to the list of captures in the same style, with
   its cost (measure it, see below) and the two bounds.

Do not change what is measured when `heap` is not asked for: a run without it must write
byte-identical records (apart from times). Show that with a before/after run.

## How to test without cargo

Find how the Rust tests start the agent (`crates/dla-workload/tests/harness_capture_python.rs`
and `harness_python_blocks.rs`: the environment variables, `PYTHONPATH`, the driver file) and do
the same from small shell or Python scripts in your scratch directory, with the machine's
`python3` (3.13) AND with
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/pillow/.venv/bin/python3
(3.12). Write a tiny project of your own in the scratch directory with these cases, each in its
own module, and a driver that calls them per unit:

- (A) a module-level dict that gets one new object per unit: expected root `module-global` with
  the module name and the name of the dict, chain of one or two steps;
- (B) a class attribute list that gets one object per unit: root `class-attribute`;
- (C) the stored-exception case: a module-level object whose `__getattr__` does `raise self.ex`;
  the unit makes a local object, touches the object inside try/except, and returns; the local
  object's line must grow and its chain must go through frame, traceback and the exception to the
  module global that holds the object;
- (D) a line whose blocks are held from native code only. Use something in the standard library
  that keeps a reference without a tracked container if you can find one; if not, say so and
  cover (d) with a unit test of the walk on a synthetic graph instead;
- (E) a line that grows by untracked objects (strings) kept in a module-level list: found through
  the referents of the list;
- (F) a line that does not grow: no entry.

For each case print the file the agent wrote and check the expectations with assertions in the
script. Measure the time and the peak extra memory of the capture on a heap of about one million
objects (make them in the test project) and report both.

## Rules

- No cargo, no commit, stash, checkout, reset, restore or branch change. Leave the tree dirty.
- No `rm -rf` on variable paths. Do not kill processes by name. Do not start any model or agent.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English in comments, full sentences, no em dashes. Never name a real
  third-party project, library or its APIs in the repository (the standard library is fine).
  No model names, no attribution lines.
- Python 3.9 syntax at most (the agent runs in the project's own interpreter): check what the
  file uses today and stay within it.
- If an action is denied, stop and report it.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: what you added (functions, constants); the file format as
written, with one real example of a site entry for case (C); each case with what the file said;
the cost numbers; the before/after check without `heap`; what the Rust side needs to know
(environment variable values, file names, the `captures` entry); what you did not verify.
