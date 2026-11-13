# jvm-session-index

Category `archetype-leak` (trap half of a pair, partner `jvm-session-index-pruned`).
Cause `L-STALE`, holder `static/global`, region `heap`.

## Mechanism

`SessionStore` keeps each open session in two static containers: `BY_ID` (the
primary store, used by every request) and `BY_USER`, a `Map<String, List<Session>>` that
exists so the operator can "sign a user out everywhere". `open` adds the session
to both (`src/portal/SessionStore.java:18-19`). `close`, called on every logout,
removes it from `BY_ID` only (`:28-30`). The per-user list keeps the closed session
and the 1.5 KB outbox and preferences it owns. `closeAllFor` tolerates the closed
entries (it skips sessions that are no longer in `BY_ID`), so nothing fails. The
number of open sessions stays at about 200; the retained set grows with the
number of logouts. The only pruning is `closeAllFor`, which removes a user's whole
list, and the driver calls it for 12 watched accounts out of 300.

Expected site: `src/portal/SessionStore.java:19` (the add to the per-user list).
Alternatives: the `BY_USER` declaration (line 11) and `close` (lines 28-30).

## Why this is `L-STALE` and not one of the UNCLASSIFIED chains

The archetype is an entry that stays in a container the program owns after the
state it describes was removed elsewhere. Here that is exact: the primary store
dropped the session, the index did not. Three other fixtures grow with the
number of units and are `UNCLASSIFIED`; this one differs from each of them:

- `jvm-audit-chain` and `node-retry-scheduler`: the retained objects reach each
  other through fields of live objects (a `previous` field), starting from one
  static or singleton reference. No collection grows and no second container
  names the objects. Here no session refers to another session; the path to each
  one is `SessionStore.BY_USER` -> `ArrayList` -> `Session`, through a container.
- `python-ingest-two-leaks` (second leak): a parser keeps each record through a
  `previous` chain from a module global, the same field chain.

A way to tell them apart in a run: remove the index entry and the object is
collectable (here), against cutting a field link in the middle of a chain
(there). The primary store stays flat in both versions of this pair.
It is also not `L-COLLECT`: the keys of `BY_USER` are bounded (300 users); the
values grow, so growth does not depend on fresh keys.

## Distinguishing edit (trap vs control)

The control's `close` also removes the session from the per-user list and drops
the list when it is empty. `diff -ru` of the two `project/` directories (23 lines
of output, 13 added or removed lines):

```
@@ -26,7 +26,18 @@
 
     /** Ends one session; returns false when it was not open. */
     static synchronized boolean close(String sessionId) {
-        return BY_ID.remove(sessionId) != null;
+        Session session = BY_ID.remove(sessionId);
+        if (session == null) {
+            return false;
+        }
+        List<Session> sessions = BY_USER.get(session.userId());
+        if (sessions != null) {
+            sessions.remove(session);
+            if (sessions.isEmpty()) {
+                BY_USER.remove(session.userId());
+            }
+        }
+        return true;
     }
 
     /** Ends every open session of a user; returns how many were open. */
```

## Measurements

Toolchain: OpenJDK 21.0.12 (the host `java`; the fixture names no `[toolchain]`).

```
cd project && ./build.sh run 8000       # compiles, runs the unit tests, replays 8000 logins
```

The driver forces a GC at each checkpoint and prints the heap in use. Separate
runs at three counts (last checkpoint of each; sessions open stays 199 or 200):

| logins | heap in use after GC |
|---|---|
| 2000  | 5333 KB |
| 8000  | 16797 KB |
| 32000 | 61601 KB |

The slope between the 8000 and 32000 runs is 1.87 KB per login, linear in the
iteration count. A 400000-login run ends at 749281 KB (1.87 KB per login), no
plateau.

Class histogram of `java -Xint -Xmx3g -cp out portal.Main 500000`, with
`jcmd <pid> GC.class_histogram` at 3 s intervals (a full GC runs first):

| sample | `portal.Session` instances |
|---|---|
| 1 | 169784 |
| 2 | 272229 |
| 3 | 377229 |

About 200 of them are open; the others are reachable only through the lists in
`BY_USER`. In the control the same histogram reads 201 at every sample.

Unit tests: 5 in `test/portal/SessionServiceTest.java`, all pass.
