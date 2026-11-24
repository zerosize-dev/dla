# Brief G: the Go census does not see unexported types as long-lived owners

Repository: /home/user/dla (Rust workspace, branch `main`). Another change may be in the working
tree when you start (uncommitted edits outside `crates/dla-census/`): leave it alone.
Scratch directory: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u111
(create it; write nothing else outside the repository).

## What happened (a real unattended run on a Go library the tool had never seen)

The library has a documented leak of this shape (names changed):

```go
// state.go
var globalProvider = defaultProvider()

func defaultProvider() *atomic.Value {
	v := &atomic.Value{}
	v.Store(providerHolder{p: &provider{}})
	return v
}

// meter.go
type provider struct {
	mtx    sync.Mutex
	meters map[key]*meter
}

type meter struct {
	mtx         sync.Mutex
	instruments []instrument   // the leak: appended in 14 methods, reset only in setDelegate
	registry    list.List
}

func (m *meter) Counter(name string) (Counter, error) {
	...
	i := &counter{name: name}
	m.instruments = append(m.instruments, i)
	return i, nil
}
```

`dla census --lang go` listed `meter.registry` (its name matches the registration vocabulary) and
did not list `meter.instruments`. Cause, read in the code:

- `crates/dla-census/src/lang/go.rs`, the `field_declaration` arm: a field of a container type
  is a `container-field` candidate only when its owner type `is_long_lived` (or is
  self-referential).
- `crates/dla-census/src/model.rs`: `ProjectInfo::note_fields` records which types an owner
  holds with `crate::lang::caps_idents`, which takes only identifiers that start with an upper
  case letter. A Go type that is not exported starts with a lower case letter, so `provider`
  holding `map[key]*meter` records nothing, and the closure in `ProjectInfo::finish` never
  reaches `meter`.
- Nothing makes `provider` long-lived either: it is built inside a function that a
  package-level `var` initializer calls, and stored through `atomic.Value`.

Most state-holding types of a Go library are unexported, so this hides a large class of
candidates.

## The change (Go only, `crates/dla-census/`)

1. Held types of any case. For Go, record the types a struct's fields mention using the set of
   type names the project declares (gather the declared type names in the pre-pass; a name of
   any case counts when the project declares a type of that name in the same package, and an
   exported name through a package qualifier as today). Builtin types, package qualifiers and
   field names must not become edges. Do not change what `caps_idents` does for the other
   languages.
2. Types built at start-up are long-lived. A project type is long-lived when a composite
   literal of it (`T{...}`, `&T{...}`) or `new(T)` appears
   - in the initializer of a package-level `var`, or
   - in the body of a function that a package-level `var` initializer or an `init` function
     calls directly (one level of call, same package), or in `init` itself.
   Read how "constructed in `main`/`init`/startup code" is recorded today (`collect` in go.rs,
   `entry_fn_name`) and extend that; say in your report what the rule was and is.
3. Nothing else changes: the same `container_type` test, the same holders and kinds. The new
   candidates are ordinary `container-field` entries with their writers and removers found by
   the existing site pass (check that `m.instruments = append(m.instruments, i)` is recorded as
   a writer and `m.instruments = nil` as a remover or reset: say what the entry looks like).

## What to measure and report

- Unit tests in `crates/dla-census/src/lang/go.rs` (next to
  `container_fields_need_a_long_lived_owner`): the shape above gives a `container-field` entry
  for `meter.instruments` with its writers; an unexported struct with a slice field that nothing
  long-lived holds still gives none; a lower-case builtin or a field name does not create an
  edge; a type built in a function that only a test calls is not long-lived. Each seen failing
  under a mutation you apply and revert.
- The count of Go census candidates before and after on every Go fixture under
  `bench/fixtures` (`ls bench/fixtures | grep -i go`): build the binary, and for each fixture
  run the census on a COPY in your scratch directory (read how `crates/dla-cli/tests` or
  `dla-bench static` run a census on a fixture; do not write into `bench/fixtures`). List the
  new candidates per fixture (owner and site). Then run
  `cargo run -j 3 -p dla-bench -- static` the way CI does (find the exact command in
  `.github/workflows/ci.yml`, job "static acceptance on the fixtures") and report pass/fail
  counts: every fixture that passed before must pass.
- Any golden or snapshot file of the census that changes: say which and why.

## Rules

- Edit only under `crates/dla-census/` (and census golden files if a test needs it; say which).
  Do NOT touch `skills/`, `agents/`, `docs/`, `hooks/`, `data/`, `harness/`, other crates.
- No `git commit`, `git stash`, `git checkout` (in any form), `git reset`, `git restore` or
  branch change. Leave the tree dirty. Undo your own edits by hand or with your mutation script.
- Shared `target/` only (`cargo` from /home/user/dla, `-j 3`). No `rm -rf` on variable paths. Do
  not kill processes by name. Do not run `dla consent`. Do not start any model, agent or
  verifier. No package installs.
- Run each cargo command on its own, in the foreground, piped through `tail` or `grep`. If the
  build fails in a crate you did not edit, another change is in progress there: wait two
  minutes and try again, up to five times, then report it. If an action is denied, stop that
  action and report it; do not ask anyone to do it for you and do not try another way to the
  same end.
- Keep to this brief. If you find another defect, report it; do not fix it.
- Plain technical English, full sentences, no em dashes. Never name a real third-party project,
  library or its APIs in the repository. No model names, no attribution lines.
- Mutations by a small script that asserts the old text occurs exactly once. Before you finish,
  `grep -rIl "MUT_" crates harness skills docs bench agents hooks data` must print nothing
  (a marker of the other change in progress is not yours: report it, do not remove it).

## Checks before you report

1. `cargo fmt --all --check` clean for the files you edited.
2. `cargo clippy -j 3 -p dla-census --all-targets -- -D warnings` clean.
3. `cargo test -j 3 -p dla-census` passes.
4. The static acceptance run described above.

## Report (final message; read by the orchestrator)

Two sentences of result first. Then: files changed; the old and the new rule for a long-lived Go
type, exactly; the census entry of the leak shape (one JSON object); candidates before and after
per Go fixture; the static acceptance counts; each test with its mutation; anything in this
brief that was wrong; what you did not verify. Counts only.
