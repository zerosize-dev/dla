# rust-private-registry

Category `impossible`. `dynamic_confirmation = false`, the leak is `confirmable = false`:
the project cannot be built without access to a private crate registry, so a run can only
suspect the cause from the source and must never confirm it.

## Mechanism (not visible at runtime here)

`parcel-relay` decodes a byte stream with the `Decoder` of the `parcel-codec` crate, which
the build takes from the private registry `acme` (`Cargo.toml`, `.cargo/config.toml`). The
decoder hands each decoded frame to the caller as a raw `*mut Frame` through `next_frame`.
`frames::drain` calls `parcel_codec::release(ptr)` for `Data` frames only
(`src/frames.rs:19`); the `Control` and `Heartbeat` arms (`:21-25`) read the frame and drop
the pointer without releasing it. Whether that retains memory depends on the crate's
ownership contract: if the decoder keeps ownership of the frames it hands out and reuses
them, the code is correct; if the caller owns every frame it receives, one frame in five is
stranded. Only the crate and its manual define which, so the behaviour cannot be settled by
replacing the crate with a stand-in.

Cause `L-OWNERSHIP`, holder `native-handle/global-ref`, region `off-heap`. Expected site:
`src/frames.rs:21` (the first arm that does not release; the range covers both),
alternatives: the point where the pointer is taken (`:14`) and the release of the data arm
(`:19`).

## Failing build

The registry host does not resolve outside the company network. In the environment where
this fixture was written, outbound connections go through a proxy that refuses the tunnel,
so the build stops at the same step (fetching the registry's `config.json`):

```
$ cd project && cargo build
    Updating `acme` index
warning: spurious network error (3 tries remaining): [7] Could not connect to server (CONNECT tunnel failed, response 403)
warning: spurious network error (2 tries remaining): [7] Could not connect to server (CONNECT tunnel failed, response 403)
warning: spurious network error (1 try remaining): [7] Could not connect to server (CONNECT tunnel failed, response 403)
error: failed to get `parcel-codec` as a dependency of package `parcel-relay v0.4.2 (/…/project)`

Caused by:
  failed to load source for dependency `parcel-codec`

Caused by:
  unable to update registry `acme`

Caused by:
  download of config.json failed

Caused by:
  [7] Could not connect to server (CONNECT tunnel failed, response 403)
```

(cargo 1.99.1, exit status 101.) With `cargo build --offline`:

```
error: no matching package named `parcel-codec` found
location searched: `acme` index
required by package `parcel-relay v0.4.2 (/…/project)`
note: offline mode (via `--offline`) can sometimes cause surprising resolution failures
help: if this error is too confusing you may wish to retry without `--offline`
```

The unit tests need the same crate and cannot run either.

## Source sanity check (not part of the fixture)

To make sure the Rust is well-formed, `project/` was built once in a copy outside the
repository against a throw-away stand-in crate (a path dependency of about 40 lines that
allocates every frame with `Box::into_raw` and frees on `release`). The 4 unit tests pass
and a 1000-packet run prints `packets=1000 frames=3447 bytes=1274655 controls=357
heartbeats=356`. Whatever memory such a stand-in retains comes from the contract its
author chose and is not evidence about the real crate; it only shows the code path that
reads a frame and does not release it. Nothing in `project/` says which contract the real
crate has.
