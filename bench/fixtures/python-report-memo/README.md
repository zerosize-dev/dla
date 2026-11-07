# python-report-memo

Category: `archetype-leak` (trap half of a pair with `python-report-memo-bounded`).
Cause `L-COLLECT`, holder `static/global`, region `heap`.

## Mechanism (for authors)

`ReportBuilder.category_totals` is decorated with `functools.lru_cache(maxsize=None)`.
The cache lives on the class (module-level, process lifetime) and its keys include
`self`. `ReportService.handle` builds a new `ReportBuilder` for every request, so each
summary request adds one cache entry that keeps the whole builder, including its
column buffers holding the last rendering, alive. Nothing evicts entries.
Detail-style requests never call the cached method, so only about 80% of requests
retain a builder.

## Minimal edit to the control

The control (`python-report-memo-bounded`) changes the decorator to `maxsize=128` and
makes the service hold one builder for its lifetime. The `diff -ru` (30 lines of
output, 5 lines removed, 3 added in 2 files):

```
diff -ru ../python-report-memo/project/reports/builder.py project/reports/builder.py
--- ../python-report-memo/project/reports/builder.py	2026-11-07 15:33:16.459296240 +0000
+++ project/reports/builder.py	2026-11-07 15:33:48.754624876 +0000
@@ -11,7 +11,7 @@
         self._currency = currency
         self._columns = {"category": [], "amount": [], "memo": []}
 
-    @functools.lru_cache(maxsize=None)
+    @functools.lru_cache(maxsize=128)
     def category_totals(self, account, month):
         totals = {}
         for row in self._store.fetch(account, month):
diff -ru ../python-report-memo/project/reports/service.py project/reports/service.py
--- ../python-report-memo/project/reports/service.py	2026-11-07 15:33:16.461517171 +0000
+++ project/reports/service.py	2026-11-07 15:33:48.754774444 +0000
@@ -7,12 +7,10 @@
     """Answers report requests from the front end."""
 
     def __init__(self, store, currency="USD"):
-        self._store = store
-        self._currency = currency
+        self._builder = ReportBuilder(store, currency)
         self.served = 0
 
     def handle(self, request):
-        builder = ReportBuilder(self._store, self._currency)
-        body = builder.render(request.account, request.month, request.style)
+        body = self._builder.render(request.account, request.month, request.style)
         self.served += 1
         return Response(request.account, request.month, body, formatting.checksum(body))
```

## Measurements

Harness outside `project/` (it drives `reports.load.requests` and calls
`gc.collect()` before each reading; `traced` is `tracemalloc` current bytes,
started after the imports):

```
cd bench/fixtures/python-report-memo/project
python3 harness.py <project dir> 2000 8000
n=2000 i=500  traced_kb=1142  builders=400  cache_entries=400
n=2000 i=1000 traced_kb=2295  builders=800  cache_entries=800
n=2000 i=1500 traced_kb=3434  builders=1200 cache_entries=1200
n=2000 i=2000 traced_kb=4609  builders=1600 cache_entries=1600
n=8000 i=2000 traced_kb=4208  builders=1600 cache_entries=1600
n=8000 i=4000 traced_kb=8812  builders=3200 cache_entries=3200
n=8000 i=6000 traced_kb=13347 builders=4800 cache_entries=4800
n=8000 i=8000 traced_kb=18032 builders=6400 cache_entries=6400
```

Growth is linear in requests: about 2.3 KB per request (4208 KB at 2k, 18032 KB at 8k,
slope 2.30 KB/request; 0.8 retained builders per request, about 2.9 KB each). Process
`ru_maxrss` was 30 MB after the 2k run and 70 MB after the 8k run.

Workload: `python3 -m reports.load --iterations 8000` takes about 1.5 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (7 tests, pass).
Size: 8 files, 254 lines.
