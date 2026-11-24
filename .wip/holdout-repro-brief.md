# Brief HR: settle two answer-key entries by running the code

Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u106
Read `u106/answer-key.md`, sections 5 (@apollo/client v3.3.13) and 6 (@mswjs/interceptors
v0.40.0). Both have the verdict UNSURE: the unfixed code is there, but reading alone did not
show that memory grows. The person who reads your final message must NOT learn where the leaks
are; they only need to know whether each one is real at that version.

Installed and built copies of both projects, made for you, are in `u106/repro/apollo-client`
(built into `dist/`, CommonJS entry points such as `dist/core/core.cjs.js`) and
`u106/repro/interceptors` (built into `lib/`, for example `lib/node/index.js`). Dependencies
are in their `node_modules`. Node is v22.

For each of the two projects:
1. Write a script in `u106/repro/scripts/` (create it) that loads the built package from the
   copy and runs the trigger from the answer key in a loop, with `node --expose-gc`. Measure
   after a forced `global.gc()` at ten or more evenly spaced checkpoints: `heapUsed`, and where
   you can reach it, the size of the object the key names as the holder.
2. Run a control beside it: the same loop used the way that should not leak (the release call
   made, or the fix from the key applied to the copy's built file; if you patch a file, keep
   the original next to it and restore it afterwards).
3. Decide: `GROWS` (a steady rise per iteration after forced collection that the control does
   not show; give the bytes per iteration in the key), `DOES NOT GROW`, or `COULD NOT RUN`
   (say why in the key).
4. Update that section of `u106/answer-key.md`: the verdict line becomes `CONFIRMED by
   reproduction`, `NOT REPRODUCED` or `UNSURE`, with the script path, the numbers, and any
   correction of the holder, the adder or the trigger that the run showed.

Limits: about twenty minutes of work per project; if the first trigger does not grow, try the
variations the issue text describes, then stop. Only `node` commands, only inside `u106/repro`.
No installs, no network. No `rm -rf` on variable paths; do not kill processes by name. Do not
touch /root/dla-real or /home/user/dla. If an action is denied, stop that action and report it;
do not ask anyone to do it for you and do not try another way to the same end.

Final message (read by the person who must stay blind): per project only the number, the name
and one of GROWS / DOES NOT GROW / COULD NOT RUN, and for COULD NOT RUN a reason that gives
nothing away. No file, function, class, mechanism or API name of either leak.
