# rust-meter-callbacks-reclaimed

Category `decoy-control` (control half of a pair, partner `rust-meter-callbacks`).
Expected answer: no confirmed leak; the `Box::into_raw` handed to the callback table
is ruled out (`N1`, `N-WARM`: live memory is flat from the first sessions).

## Mechanism

Identical to the partner except that `Session::close` takes the pointer that
`Registry::remove` returns and converts it back with `Box::from_raw`, so the
`Window` is dropped when the session ends. The raw-pointer hand-off to a C-style
callback table is as suspicious as in the trap.

## Distinguishing edit (trap vs control)

`diff -u` of `src/session.rs`, run from `bench/fixtures`: 3 added lines, 1 removed.

```
@@ -30,7 +30,9 @@
     }
 
     pub fn close(self, registry: &mut Registry) {
-        registry.remove(self.id);
+        if let Some(user) = registry.remove(self.id) {
+            drop(unsafe { Box::from_raw(user as *mut Window) });
+        }
     }
 }
 
```

## Measurements

Same harness as the partner (counting allocator, `VmRSS` at the end of the run):

| sessions | active entries | live bytes | live blocks | RSS |
|---|---|---|---|---|
| 2000   | 0 | 746 | 7 | 2104 kB |
| 8000   | 0 | 746 | 7 | 2044 kB |
| 32000  | 0 | 746 | 7 | 2108 kB |
| 128000 | 0 | 746 | 7 | 2108 kB |

Live bytes are the same at every count, and the slope of the second half is zero.
Contrast with the trap: 533 248 748 live bytes and 528 104 kB at 128000 sessions.
Unit tests: `cargo test --release` (5 pass).
