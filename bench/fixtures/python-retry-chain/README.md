# python-retry-chain

Category: `archetype-free`. Cause `UNCLASSIFIED`, holder `static/global`, region `heap`. No pair.

## Mechanism (for authors)

`Sender.deliver` retries a delivery until the endpoint accepts it. Each failed pass
raises a `RetryError` and records it in `health._last_failure[endpoint]` (a module-level
dict) for the status page. The raise is written `raise RetryError(...) from (previous or
exc)`, where `previous` starts as the failure the status page already holds for that
endpoint (`health.last_failure`) and then becomes the failure just raised. So every
`RetryError` has the previous one as its `__cause__`, across passes and across
deliveries. The dict holds one exception per endpoint (12 endpoints), but that one
exception reaches every earlier one through `__cause__`. Each exception also keeps its
traceback, which keeps the `deliver` frame of its call alive with its locals (the encoded
`body`, the `delivery`).

No collection, cache, listener, timer, thread or captured scope grows: the retained
objects are exceptions, frames and byte strings, reached through the exceptions' own
attributes from one module-level dict entry. That is why no taxonomy row fits
(`L-COLLECT` needs a growing container; the dict here has a fixed number of keys), so the
expected label is `UNCLASSIFIED`.

Minimal edit that removes the growth: `from exc` instead of `from (previous or exc)`.
Retained exceptions drop to one per endpoint (see below).

## Measurements

Harness outside `project/` (drives `dispatch.load.deliveries` through
`DeliveryService`, `gc.collect()` before each reading, `tracemalloc` started after the
imports, one fresh process per count):

```
python3 -B harness.py <project dir> 2000
n=2000 i=500  traced_kb=1155 retry_errors=651
n=2000 i=1000 traced_kb=2265 retry_errors=1271
n=2000 i=1500 traced_kb=3373 retry_errors=1877
n=2000 i=2000 traced_kb=4429 retry_errors=2449
python3 -B harness.py <project dir> 8000
n=8000 i=2000 traced_kb=4429 retry_errors=2449
n=8000 i=4000 traced_kb=8781 retry_errors=4837
n=8000 i=6000 traced_kb=13281 retry_errors=7298
n=8000 i=8000 traced_kb=17641 retry_errors=9664
```

Growth is linear in deliveries: 2.2 KB per delivery (4429 KB at 2k, 17641 KB at 8k, slope
2.20 KB/delivery) and about 1.2 retained `RetryError` per delivery. `ru_maxrss` was 26 MB
after 2k and 54 MB after 8k. With `from exc` the same harness reads 35 KB and 12
`RetryError` objects (one per endpoint) at both 2k and 8k, `ru_maxrss` 18 MB.

Python 3.13.18. Workload: `python3 -m dispatch.load --iterations 8000` takes about 0.2 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (8 tests, pass).
Size: 10 files, 264 lines.
