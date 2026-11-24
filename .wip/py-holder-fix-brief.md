# Brief R3: defects found when the holder evidence was run on a real library

Same repository, same rules as brief R2 (`py-holder-rust-brief.md`, section "Rules"), same scratch
directory (`u93`). Your R2 work is in the tree and stays; this round corrects it. I ran the frozen
binary on a real Python library with C extension modules that is installed in editable mode: one
hook driver whose unit makes two leaks at once (a native reference leak under one line, and a
module-level object that re-raises a stored exception). 440 units per round, 9 points. What I saw
is below, most important first. Each item says what must be true afterwards; how is yours.

## 1. The walk does not finish (agent, `harness/python/sitecustomize.py`)

The chain to the module global runs through about 16 000 traceback objects. The capture took
60.5 s, the time bound; the file says "the time bound (60 s) stopped the walk: 162 kept objects
have no chain". Of six growing lines only the first got chains (38 objects in 60 s, about 1.5 s per
object); the other five have `kept: 40` and `chains: []`. Every kept object's chain runs into the
same long list, and each walk goes through it again.

After: on a fixture of your own with a stored exception raised at least 16 000 times and six
growing lines (40 kept objects each), the whole heap capture takes under 10 s on this machine and
every line has chains; no limitation about the time bound. A walk that reaches an object whose
way to a root is already known stops there and reuses it. The chains written (types, `via`, `at`,
`repeat`, roots, counts) are the same as before for the cases A to I in scratch `u91`
(`python3 u91/check_cases.py <python> <tag>` must still print ALL OK under 3.13 and under the
3.12 of `u72/pillow/.venv/bin`). Test: one Python-agent test in
`crates/dla-workload/tests/harness_capture_python.rs` (or the file that fits) with a bound on
the capture's own time that the old walk misses by a wide margin; state both times in the report.

## 2. `at` is an absolute path cut in the middle (agent)

For a library installed in editable mode the frames run from the source tree, not from the copy
(`DLA_SOURCE_ROOT`). The growing lines are named relative to the project (`src/pkg/cms.py:411`),
but a step's `at` came out as `...273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u72/<lib>/src/pkg/cms.py:176`.
`next` then printed `dla ledger add ... --site ...273-1d53-.../src/pkg/cms.py:176`, the bundle
could not excerpt the frame, and a local path went into evidence.

After: `at` (and the `at` of a frame root) is named by the same rule as the growing lines
(same function), in the copy and in the source tree alike; a file outside the project keeps a
short neutral form that is not an absolute path. Test with a project run from its source root
(the existing tests of the editable-mode case show how).

## 3. A bundle with a retention record no longer needs the stack (`bundle_cmd.rs`, `stacks_check`)

For the finding at the native line (holder entry `unreferenced`, no path) `dla bundle` was accepted
with evidence E7 (trend), E9 (retention), E12 (alloc-test) and no allocation stack. Before R2 the
same state was refused with `stacks-not-captured`. `stacks_check` returns early when the bundle
holds a record of class Allocation or Retention; a Python retention record now exists, so the
check is off exactly where the stack is the only thing that shows the verifier the line and the
call into the extension.

After: a retention record lifts the stack requirement only when one of its holder entries that
hold the finding's line (or are owned by its owner) has a `path`. An `unreferenced` entry does
not. Test: the state above is refused with `stacks-not-captured`, and with a real chain it is
accepted.

## 4. A capture summarized before the finding existed

The new `holders_hint` of `dla trend` leads to: capture, both summaries, then
`dla ledger add` from the `next` of the heap-graph summary. The allocation summary was then
written before the finding existed and names no finding. Afterwards `dla alloc-test` printed
`dla harness run H1 --mode fresh --capture alloc,heap` again (a second capture of a minute), and
`stacks_check` would say "was summarized and no growing stack ends at the finding's line", which
is false: the stack ends there.

After: a stack summary made before the finding was added counts for the finding once it exists,
when one of its growing stacks has the finding's line as its first project frame: `dla bundle`
holds that record and its stack with no second summarize and no second capture, and
`dla alloc-test` does not print a capture command when such a record exists (its `next` goes
straight to refute and confirm, and `stacks_hint` is absent). Also change `holders_hint` to
`--capture alloc,heap` and "the two `dla summarize` commands it prints", so one capture run serves
both. Tests: the order trend, capture, summaries, add, sibling, test, alloc-test, refute, confirm,
bundle on a real Python fixture ends with a bundle that holds the stack, the retention record and
an excerpt of the finding's line.

## 5. The bundle shows the wrong holder

The retention record of that capture has two holders: the module global with its chain (it keeps
other lines) and the native entry of the finding's line. The bundle of the finding at the native
line carried the whole record: the first holder a verifier reads is the module global with a full
path, the entry that is about the finding has key and site `<redacted>`, and `excerpt.txt` showed
four lines of the other leak (taken from the raw file's site list) and not the finding's line.

After: the copy of a retention record in a bundle (the `result` in `index.json` and the raw file)
holds only the holder entries that hold the finding's line or are owned by its owner, cut before
the scrub and before frames are collected; the number of holders left out is stated
(`other_holders`). Excerpt frames come only from what is left. Test on a record with two holders:
each finding's bundle shows its own holder only, and no frame of the other.

## 6. Lines and type counts the holder keeps are still "tied to no finding"

With the finding at the module global added, the final gate still asked for a finding at three
more lines (they had no chain because of item 1; check again after it) and at four type-count
columns: `py.type:<project class>.count` (two), `py.type:frame.count`, `py.type:traceback.count`.
All four are objects of the same chain.

After: the record states, per holder, the types of the objects it keeps (`types`: the type of
the kept object of each chain, the first step, and the type of a step that has `repeat`; not the
types of the other steps, so a `dict` or a `list` that is only a link in a chain is not listed;
clipped like the other lists, most frequent first), and a non-flat
`py.type:<T>.count` column that is tied to no finding by today's rules is tied to the findings
that the record names for a holder that lists `T` (same rule as for a held line; a sibling never
ties). On the real fixture of your end-to-end test the final gate then has no item that asks for
a finding at any column of the series except the whole-process ones.

## 7. A holder that keeps a few objects of a line takes the whole line

Any holder entry that lists a site ties that site's column to the holder's finding, also when it
keeps 2 of 40 sampled objects, and also when 5 objects were found for a line whose block count
grew by 1000 (the rest belongs to no object: native).

After, in the adapter: a holder holds a site (key `sites`, which ties) only when its chains carry
at least a quarter of the objects sampled for that site; else the site is listed under
`minor_sites` (no tie, no `next`). And a site for which the objects found (tracked plus
untracked) are fewer than a tenth of its block growth (growth of at least 20 blocks) gets the
`unreferenced` native entry as well, and Python holders list it under `minor_sites`. Constants
with names; unit tests for both sides of each bound.

## 8. One dump point: `blocks_first` is null and the adapter refuses the file

The agent writes `"blocks_first": null` when there was one dump point (command mode). The adapter
answers `malformed-artifact: invalid type: null, expected u64`. After: the file is read; the
record shows `first` as null and says in a limitation that growth was not computed; ties work the
same. Test with a hand-written file.

## 9. Small things

- The command output of `dla summarize heap-graph` and the record say when sites have kept
  objects and no chain (the time bound or another limit): how many sites, and the reason from the
  file's limitations. Today four lines vanished without a word.
- `harnesses[].is` in the bundle index says "a driver for other targets" for a plain driver that
  has no target at all. Say "a driver with no target" there; keep the other three wordings.
- Update the texts you wrote in R2 where an item above changes what they say (`python.md`,
  `stages.md`, `agents/verifier.md` if `types` or `minor_sites` need a word, the README of the
  agent, the VERIFY row). Pins in `skill_text.rs` follow.

## Tests and checks

Every new or changed behaviour above has a test seen failing under a mutation you apply and
revert (script, exactly-one match, `MUT_` grep empty at the end). Run the checks of brief R2
again in full, plus `python3 u91/check_cases.py` under both interpreters, plus the Python tests
under the 3.12 venv as well (`PATH=<scratch>/u72/pillow/.venv/bin:$PATH` for
`harness_capture_python`, `harness_python_blocks`, `py_site_columns`, `summarize_python`); two
`gc.freeze` tests of `harness_capture_python` are known to fail under that 3.12 and are not yours.

Do not run anything in `u72/pillow` or `u101` (my check runs live there).

## Report

Two sentences of result first. Then per item 1 to 9: what changed, the test, the mutation. Then
existing tests changed and why; anything in this brief that is wrong; what you did not verify.
