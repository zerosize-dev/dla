# Integration suite

The suite in `crates/dla-it` runs `dla` inside microVMs. It uses
[rightsize](https://crates.io/crates/rightsize) with the microsandbox (msb)
backend only. No Docker provider is registered, and every guest is started with
`require_isolation(true)`, so a host without hardware virtualization cannot fall
back to a container.

The suite is a first step. It answers what the host-side tests cannot: what the
guest kernel offers, whether the static binary runs on a stock image, and what
happens when analyzers are really absent.

## What runs where

| Where | Runs |
|---|---|
| `cargo test --workspace` (all OSes) | Only the unit tests of `dla-it` (quoting, parsing, fact logic). The VM stack is not compiled. |
| CI job `it-linux` (ubuntu-latest, KVM) | The VM tests. |
| CI job `it-windows` (windows-2025, WHP) | The VM tests, one at a time. Not required (`continue-on-error`). |
| macOS | No VM tests. Hosted macOS runners cannot nest VMs. |

CI builds the static `x86_64-unknown-linux-musl` `dla` once (job `it-build`) and
passes it to both VM jobs. No model call and no credential is used.

## Run it locally

You need a host that can boot microVMs (Linux with `/dev/kvm`, or Windows with
WHP) and a musl `dla`:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --locked -p dla-cli --target x86_64-unknown-linux-musl
RIGHTSIZE_BACKEND=microsandbox DLA_IT_OUT=/tmp/dla-it-out \
  cargo test -p dla-it --features sandbox-it -- --nocapture
```

The first run downloads the msb runtime and pulls `debian:bookworm-slim`.

| Variable | Meaning |
|---|---|
| `DLA_IT_SKIP_VM=1` | Every VM test prints a notice and passes without booting. It wins over `DLA_IT_REQUIRE_VM`: with both set, the tests skip. |
| `DLA_IT_REQUIRE_VM=1` | A host that cannot boot a microVM fails the test. CI sets it and never sets `DLA_IT_SKIP_VM`. Without it the test skips. |
| `DLA_IT_BIN_DIR` | Directory with the static `dla`. Default: `target/x86_64-unknown-linux-musl/release`. |
| `DLA_IT_OUT` | Directory for recorded facts (`kernel-facts.json`). Unset: nothing is written. |
| `RIGHTSIZE_BACKEND` | Set to `microsandbox` in CI. |

To run one test: `cargo test -p dla-it --features sandbox-it --test kernel`.

## Tests

### `tests/kernel.rs`: `kernel_probe_records_facts` (A1)

Boots `debian:bookworm-slim` and reads the guest kernel: kernel version, user
id and capabilities, BTF, `kallsyms`, tracefs (mounted by the script), syscall
tracepoints, writable `kprobe_events` and `uprobe_events`, cgroup v2,
`ptrace_scope`, `perf_event_paranoid`, `unprivileged_bpf_disabled`, memory.

It asserts only what any Linux guest has: the kernel is Linux, `/proc` is
mounted, the user id is numeric, and every fact the later tests branch on is
readable. It does not require BTF or tracefs. The facts go to
`$DLA_IT_OUT/kernel-facts.json` and into the log. The derived `bpftrace_ok`
field (BTF, syscall tracepoints and root) tells later tier 3 tests what to
expect. If the guest kernel lacks BTF, `dla attach bpftrace` cannot work there,
and the result is a recorded fact and a row in `docs/VERIFY.md`.

### `tests/smoke.rs`: `static_dla_runs_init_and_probe_in_a_stock_image`

Copies the static `dla` and `data/` into a stock image with no Rust, Git or
analyzers. It asserts that the binary is statically linked (`ldd` reports
"not a dynamic executable", or "statically linked" for the static-PIE musl build), `dla --version` runs, `dla run init` creates a run
with state and ledger files at stage `intake`, and `dla probe` completes, finds
no runtime in an empty project, and reports a `privilege.root` value equal to
what `id -u` says.

### `tests/coverage_gap.rs`: `missing_go_tools_become_coverage_rows`

A Go module in a stock image that has no Go. It asserts that `dla probe`
detects the Go runtime, lists `go:staticcheck`, `go:golangci-lint` and
`go:govet` as missing, and that `coverage.json` records `staticcheck` and
`golangci-lint` as `missing` with the hint `dla install <name>`, and
`bcc-memleak` as `blocked`.

## Limits of the backend that shape the code

- `exec` takes no environment, working directory or stdin, and stops after 120
  seconds. Every command runs through `sh -c`, with the environment set in the
  command line. Longer work must run as a background job and be polled.
- `/tmp` in a guest is a tmpfs. Anything a checkpoint must keep goes elsewhere.
- Image `ENV` handling for `exec` is not documented. The first CI runs show
  whether `PATH` from the image reaches the tests.

## Planned next

The rest of the suite (tool installs, version matrix, attach to a process `dla`
did not start, dump preflight under memory caps) is in a later phase. See
`docs/VERIFY.md` section "M7" for open facts the first CI runs must settle.
