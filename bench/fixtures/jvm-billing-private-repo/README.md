# jvm-billing-private-repo

Category `impossible`. `dynamic_confirmation = false`, the leak is
`confirmable = false`: the project cannot be built in a sandbox without access to
the private repository, so a run can only suspect the cause from the source.

## Mechanism (not visible at runtime here)

`ChargeService.charge` registers each charge in the static map
`InFlightCharges.AWAITING` (keyed by the ledger ticket ID) before handing it to
the ledger client. `ChargeListener.onPosted` removes the entry; `onRejected` and
`onExpired`, which the ledger library also delivers, only notify the customer.
Every charge that ends in one of those two callbacks stays in the map. Which
callbacks fire, and how often, is decided by the library's `LedgerClient`
(`newTicket`, `submit`, `PostingListener`), which is why the behaviour cannot be
reproduced by replacing the library with a stand-in.

Cause `L-COLLECT`, holder `static/global`, region `heap`. Expected site:
`src/main/java/billing/InFlightCharges.java:13` (the `put`); alternatives: the
declaration at line 8 and the registration call at `ChargeService.java:26`.

## Failing build

The only dependency, `internal.billing:ledger-client:3.2.1`, is published to
`https://maven.billing.internal.example/releases`, which does not resolve:

```
$ cd project && mvn -q package
[ERROR] Failed to execute goal on project charge-gateway: Could not collect dependencies for project example.billing:charge-gateway:jar:1.8.0
[ERROR] Failed to read artifact descriptor for internal.billing:ledger-client:jar:3.2.1
[ERROR] 	Caused by: The following artifacts could not be resolved: internal.billing:ledger-client:pom:3.2.1 (absent): Could not transfer artifact internal.billing:ledger-client:pom:3.2.1 from/to billing-internal (https://maven.billing.internal.example/releases): maven.billing.internal.example: Name or service not known
```

(Maven 3, JDK 21, exit status 1; the `target/` directory is removed afterwards.
The unit tests need the same library and cannot run here either.)

## Source sanity check (not part of the fixture)

To make sure the Java is well-formed, it was compiled once against a throw-away
stand-in for the library kept outside the repository (about 10 one-line classes
and a loop that invents its own rejection pattern). It compiled. Whatever count
of waiting entries such a stand-in produces comes from the pattern the
stand-in's author chose and is not evidence about the real library; it only
shows the code path that adds without removing. Nothing in `project/` says which
callbacks the real library fires or how often (the amounts in `Main.java` are
an ordinary spread with no special value).
