# Brief HV: check an answer key against the code, without giving the answers away

You read code and web pages only. You install nothing, build nothing and run no test, no
package manager and no program of any of the projects. You change nothing in /home/user/dla.

Scratch directory (exists):
/tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u106
Write only there. Put checkouts under `u106/src/<name>` (some may exist already; reuse them).

## Background

`u106/candidates.md` lists ten open-source projects, each at a version that is said to contain
one documented memory leak. `u106/answer-key.md` says where each leak is. A leak-finding tool
will be run blind on some of these projects, and the key is used to score the runs afterwards.
The person who runs the tool must NOT learn the answers before the runs. The key was written
mostly from issue and pull-request text: for eight of the ten the code at the leaky version was
not read. A wrong key would make a run worthless, so the key has to be checked first.

## What to do, for each of the ten projects

1. Get the code at the leaky version: `git clone --depth 1 --branch <tag> <url> u106/src/<name>`
   (for the one project given by a commit, a normal clone and `git -C ... checkout <commit>`).
2. Read the fix (the pull request or commit named in the key; a GitHub pull request has a plain
   diff at `<pr url>.diff`, a commit at `<commit url>.diff`).
3. Read the code at the leaky version and decide:
   - Is the code that the fix changes present at the leaky version, in the unfixed form?
   - Which object keeps the memory (file and line at the leaky version), and which line adds to
     it? Is there really no removal on the path the issue describes?
   - Can the growth be reached in one process by calling the project's public API in a loop,
     with no network peer, database or other service that is not part of the project? Write the
     shortest call sequence you believe triggers it (API names, not a tested program).
4. Rewrite that project's section of `u106/answer-key.md` in place so that it holds, checked
   against the code: links to the issue and the fix; the fixed version; holder file:line and
   adder file:line at the leaky version; the mechanism in two or three sentences; the trigger
   call sequence; every other file:line a correct report could reasonably name instead (for
   example the caller that never releases, or the type's definition); and your verdict:
   - `CONFIRMED`: you read the unfixed code at the leaky version and the mechanism holds.
   - `UNSURE`: say exactly what is missing.
   - `ABSENT`: the leak is not in this version (say why).
   Add a line `Checked by reading the code at the tag: yes`.

If a project turns out ABSENT at its version but a nearby release clearly has it, say which in
the key and in your report as "version should be <tag>", without saying why.

## Your final message (it is read by the person who must stay blind)

It must contain NO file name, function, class, mechanism, issue number or link of any leak.
Only, per project: number, name, version, verdict (CONFIRMED / UNSURE / ABSENT), "reachable in
one process: yes/no/unsure", and for UNSURE a reason that gives nothing away (for example "the
fix could not be read" or "the code differs from what the fix changes"). Then one line on
anything you could not do.

## Rules

Plain English, full sentences. No `rm -rf` on variable paths. Do not kill processes by name.
No installs, no builds, no tests, no `pip`, `npm`, `cargo`, `go`, `mvn`, `gradle` commands:
`git clone`, `git fetch`, `git show`, `git log`, `git diff`, file reading, text search and web
pages are all you need. If an action is denied, stop that action and report it; do not ask
anyone to do it for you and do not try another way to the same end.
