# jvm-session-index-pruned

Category `decoy-control` (partner `jvm-session-index`). Cause `N-WARM`: the
per-user index holds only open sessions, so it fills to the open-session count
and stays there.

## Mechanism

Same code as `jvm-session-index`, except that `SessionStore.close` also removes
the closed session from the per-user list and drops the list when it is empty.
Heap in use rises while the first 200 or so sessions open and is flat after
that. A run that reports a confirmed growth here is a false alarm. Both
fixtures share the same `task.prompt`.

Non-leak site: `src/portal/SessionStore.java:11` (the `BY_USER` declaration, the
container a reader suspects first).

## Minimal edit from the trap

See the `diff -ru` in `jvm-session-index/README.md`: one method, 13 added or
removed lines.

## Measurements

Toolchain: OpenJDK 21.0.12.

```
cd project && ./build.sh run 8000
```

| logins | sessions open | heap in use after GC |
|---|---|---|
| 2000  | 199 | 1862 KB |
| 8000  | 199 | 1862 KB |
| 32000 | 199 | 1862 KB |

A 400000-login run also ends at 1862 KB; the slope of the second half is zero.
`jcmd <pid> GC.class_histogram` on `java -Xint -Xmx3g -cp out portal.Main 500000`
shows 201 `portal.Session` instances at three samples.

Unit tests: 5 in `test/portal/SessionServiceTest.java` (the same file as the
trap), all pass.
