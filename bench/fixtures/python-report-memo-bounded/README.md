# python-report-memo-bounded

Category: `decoy-control` (control half of a pair with `python-report-memo`).
Cause of the look-alike: `N-WARM`.

## Mechanism (for authors)

Same code as `python-report-memo` except that the service owns a single
`ReportBuilder` and the memoised method is limited with `lru_cache(maxsize=128)`.
The grep-visible pattern (a memoised method keyed on `self`) is still present, but
there is one `self`, and the cache evicts past 128 entries, so memory rises to a
plateau within the first few hundred requests.

## Minimal edit from the trap

`diff -ru ../python-report-memo/project project` (30 lines of output, 5 lines
removed, 3 added in 2 files):

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

Same harness as the trap. Distinct keys: 4000 accounts x 12 months, so the cache
fills and then evicts.

```
python3 harness.py <project dir> 2000 8000
n=2000 i=500  traced_kb=145 builders=1 cache_entries=128
n=2000 i=1000 traced_kb=146 builders=1 cache_entries=128
n=2000 i=1500 traced_kb=145 builders=1 cache_entries=128
n=2000 i=2000 traced_kb=146 builders=1 cache_entries=128
n=8000 i=2000 traced_kb=65  builders=1 cache_entries=128
n=8000 i=4000 traced_kb=70  builders=1 cache_entries=128
n=8000 i=6000 traced_kb=72  builders=1 cache_entries=128
n=8000 i=8000 traced_kb=75  builders=1 cache_entries=128
```

Plateau: one builder and 128 cache entries at every checkpoint. The 2k run starts
higher than the 8k run because first-use allocations of the interpreter land inside
the first run's tracing window; within each run the series is flat (the 8k run
drifts by 10 KB over 6000 requests, under 0.2% of the trap's growth over the same
span). Process `ru_maxrss` stayed at 17 MB across both runs.

Workload: `python3 -m reports.load --iterations 8000` takes about 1.5 s.
Tests: `python3 -m unittest discover -s tests` from `project/` (7 tests, pass).
Size: 8 files, 252 lines.
