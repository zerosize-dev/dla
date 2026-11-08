# Test data

`journal.hprof.gz` is a heap dump of a minimal JVM running `Journal.java` (next to this
file), which keeps 2000 entries in the static list `ledger.Journal.ENTRIES` and holds a
64 KiB array only through a `WeakReference` (`ledger.Journal.SCRATCH`). The tests expect
the list to dominate 6002 objects (the list, its array and 3 objects per entry).

How it was made (OpenJDK 21, SerialGC; any JDK 17 or newer works):

```
mkdir -p src/ledger out && cp Journal.java src/ledger/
javac -d out src/ledger/Journal.java
java -Xmx16m -Xms16m -XX:+UseSerialGC --limit-modules java.base -cp out ledger.Journal 2000 &
jcmd <pid> GC.heap_dump journal.hprof        # after it prints "ready 2000"
gzip -9 -n journal.hprof
```

`--limit-modules java.base` keeps the baseline JVM heap small (2.2 MB raw, about 0.6 MB
gzipped). The file was gzipped with the system `gzip -9`, so it also exercises the
reader's inflate on real data.
