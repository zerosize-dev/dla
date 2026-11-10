# Known leaks

A row match opens a hypothesis and carries no verdict.
Each row is documented behavior of a runtime or library that looks like a leak or hides one.
`dla probe` checks the rows against the project and writes a `true`, `false` or `maybe` verdict
to probe evidence. Cite a verdict when choosing what to test; it never confirms or refutes a finding.
This file is generated from `data/known-leaks.toml`; edit the table, not this file.

| Id | Runtime | Library | Versions | Archetype | Holder | Effect | Remedy | Source |
|---|---|---|---|---|---|---|---|---|
| `go-timer-unstopped-before-1.23` | go | go | `<1.23` | L-REGISTER | registration-table | Before Go 1.23 an unstopped Timer is not collected until it fires, and an unstopped Ticker is never collected. A loop that makes one per pass grows the runtime timer heap. | Call Stop on every Timer and Ticker (defer t.Stop()), or raise the go line in go.mod to 1.23.0 or later. | https://go.dev/doc/go1.23 |
| `go-timer-asynctimerchan-revert` | go | go | `>=1.23` | L-REGISTER | registration-table | GODEBUG asynctimerchan=1 restores the old timer behavior even when go.mod names Go 1.23.0 or later, so unstopped Timers and Tickers are not collected again. | Remove asynctimerchan=1 from the godebug lines and the GODEBUG environment, or stop every Timer and Ticker. | https://go.dev/doc/go1.23 |
| `jvm-netty-leak-detection-sampling` | jvm | netty | `*` | L-HANDLE | native-handle/global-ref | Netty's default leak detection level (SIMPLE) checks about 1% of buffers, so a missing LEAK log line does not show that buffers are released. | Run the workload with -Dio.netty.leakDetection.level=paranoid, which checks every buffer. | https://netty.io/wiki/reference-counted-objects.html |
| `jvm-spring-simple-cache` | jvm | spring-boot | `*` | L-COLLECT | container-field | With no cache library Spring Boot configures the simple provider, a ConcurrentHashMap cache store that the documentation calls not really recommended for production. A ConcurrentHashMap has no size bound or expiry of its own, so entries stay until evicted by hand. | Set a bounded provider such as Caffeine with maximumSize and expireAfterWrite, or evict explicitly. | https://docs.spring.io/spring-boot/reference/io/caching.html |
