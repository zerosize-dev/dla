# Brief M: a second look takes its sequences from the key (real run tokio-3)

Rules as in brief L (Sonnet only; no commits; shared `target/`, `-j 2`; real plugin runs are
measuring on this machine: no `dla-bench`, no long loops; small exact edits; literal paths in
`rm`; never delete build directories under `target/` yourself, use
`<sp>/wip-notes/prune-superseded.sh` when disk is short; unique-marker mutations; never
`dla consent`). Scratch `<sp>/u52/`, report `<sp>/u52/report.md`, short final message.
`<sp>` = /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad

## What happened (evidence `<sp>/u36/tokio-3/`: run.json, transcript.jsonl, `dla/`)

Real plugin run on tokio-util 0.7.1 with your second look in the build. The second look
worked as built: after the first pass the tool asked for it, refused the missing
`VARIES.txt` lines and refused the copied hooks; the model wrote two second-look units; the
run ended COMPLETE in 226 seconds, "second look 2 (grew 0)", 74 of 77 budget minutes unused.

The second-look unit (H4) built a three-level chain and cancelled, but in one fixed
sequence for every unit: `c1 = root.child_token(); c2 = c1.child_token(); ...; c1.cancel();
await c2.cancelled(); drop`. The documented leak needs another sequence of the same
operations (a grandchild whose middle token is dropped at once, then the first token
cancelled). 3520 units ran the same case 3520 times. `VARIES.txt` said "caller varies the
child token created from a shared parent".

A second look that repeats one hand-picked sequence is one more sample of a space the model
cannot guess. The unit must walk the space: let the key choose the sequence.

## Wanted

1. Every driver template (Node, Python, Go, JVM, Rust) offers one small, deterministic
   helper that takes independent choices from the key: choice number `slot` out of `n`
   options (the digits of the key in mixed radix, so all combinations of the choices occur
   within a few hundred units and repeat evenly). Name it once in the natural style of each
   language and document it in two lines above the unit hook. It must not allocate or keep
   state (it runs in every unit of a measured loop).
2. The second-look block of the driver file (your to-dos) tells the model to write the unit
   as choices and gives the recipe, in general terms, for the target's owner type:
   - how deep to derive when objects of the type are made from objects of the same type
     (1 to 3);
   - which object of the chain gets the ending call (cancel, close, abort, shutdown, remove,
     unsubscribe: none, the first, a middle one, the last);
   - when the intermediate objects are released: at once (a temporary), before the ending
     call, after it;
   - the order of the remaining releases;
   - the success or the failure path where the API has one.
   One sentence says why: a fixed sequence runs one case for every unit.
   If the census gives you the owner type's own operations cheaply (methods of the type that
   return the same type, methods named like an ending call), list them by name in the
   block; if that needs new census work, do not start it: say so in the report.
3. `dla harness run` refuses a second-look harness whose unit hook does not take at least
   two choices from the key with the helper (a check of the hook text is enough: say what
   it checks), with a fix line that points at the recipe. The existing refusals (hooks equal
   to an earlier harness, `VARIES.txt`) stay.
4. The run says how much of the space was walked: the driver counts the distinct
   combinations of choices its units took (cheaply: the product of the `n` values seen per
   slot is enough if a true count costs allocation) and the run output and `dla trend` carry
   it for a second-look harness in one short line ("the units took 72 combinations of 3
   choices").
5. The first-pass hint gets one sentence that the helper exists and that order and depth
   should come from the key. No refusal in the first pass.

## Tests and checks

- Per template: the helper gives every combination of two slots within n1*n2 keys and is
  stable (unit or template test, in the style the templates are tested today).
- Integration (the `second_look.rs` style): a second-look unit without the helper is
  refused with the fix line; with two choices it runs and the combination line appears;
  each seen failing once (unique marker, exact reverse).
- Real tree: copy `<sp>/u18/tokio/proj` to `<sp>/u52/tokio` (never work in the original);
  init with the unattended-runner environment, probe, sweep, census; first pass with the
  unit of the real run's H1/H2 (`<sp>/u36/tokio-3/dla/harness/*/driver/dla_harness.rs`);
  then the second look for the cancellation-token candidates: write its unit ONLY from what
  the generated block says (choices for depth, which token is cancelled, when the middle
  token is dropped, drop order), not from knowing the documented sequence, and say in the
  report exactly what you wrote and which line of the block led to each choice. Run, trend:
  report `native.live_bytes`, the combination line, what trend prints next, and how far the
  printed commands carry (`--capture alloc`, `summarize alloc-stack-diff`, sibling,
  `ledger test`, `alloc-test`, refute, confirm). Stop and report at the first command that
  is refused or wrong.
- fmt; clippy on what you touched; `cargo test -j 2 -p dla-core -p dla-workload -p dla-cli`
  (`--no-fail-fast`; name failures that are not yours); `dla schema --check`; skill text
  test and docs where the second look is described; `docs/VERIFY.md` rows.

Report: what changed, tests with what was seen failing, the real-tree outputs, what is
unverified, anything wrong you saw outside this brief.
