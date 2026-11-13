# node-native-addon

Category `impossible` (`dynamic_confirmation = false`, the leak is
`confirmable = false`). Expected answer: `suspected_static_only` or `unresolved` with a
reason; never confirmed. The cause the source suggests is `L-REGISTER`, holder
`registration-table`, region `non-heap`, at `src/session.js:14` (alternatives
`src/session.js:25-27`, `src/relay.js:23`).

## Mechanism (not visible at runtime here)

`Session` registers a callback with the native frame codec for its device
(`codec.subscribe`, `src/session.js:14`) and keeps the function that the call returns
in `this.unsubscribe`. `Session.close()` only sets a flag; it never calls
`this.unsubscribe`. `Relay.disconnect` calls `close()` and drops the session. Whether
the codec's native table keeps the callback (and the closure that reaches the session,
the store and the counters) after the session is gone depends on the addon's
contract: whether `subscribe` holds a strong reference, whether the table is cleared
when the device disconnects, whether the callback is released when the returned
function is not called. All of that lives in the addon, which cannot be installed
here, so no stand-in can settle it.

## Failing build

The only dependency, `@meridian/frame-codec@3.1.0` (a native addon), is published to
the private registry named in `project/.npmrc`
(`https://npm.meridian.internal.example/`). In the environment this fixture was
built in, all outbound traffic goes through a proxy that answers 403 for that host
(on a host without the proxy the name does not resolve):

```
$ cd project && npm install
npm error code E403
npm error 403 403 Forbidden - GET https://npm.meridian.internal.example/@meridian%2fframe-codec
npm error 403 In most cases, you or one of your dependencies are requesting
npm error 403 a package version that is forbidden by your security policy, or
npm error 403 on a server you do not have access to.
```

and without the package the program does not start:

```
$ node load.js
Error: Cannot find module '@meridian/frame-codec'
Require stack:
- .../src/session.js
- .../src/relay.js
- .../load.js
```

`node --test` runs `test/frames.test.js` (3 tests pass: it needs no addon) and fails
`test/relay.test.js` for the same missing module. The directory is left without
`node_modules` and without a lock file.

## Source sanity check (not part of the fixture)

The JavaScript was run once against a throw-away stand-in for the addon kept outside
the repository (three lines: a `Map` of callbacks per device and a decoder that
calls them): all 6 tests pass and `node load.js` runs. Whatever the stand-in retains is the stand-in author's choice and is not
evidence about the real addon; it only shows that the code path registers a callback
for every connection and never calls the function it gets back.
