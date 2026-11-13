package exports;

import java.util.ArrayDeque;

/** Load driver: runs export jobs while the number of jobs in flight builds up like a morning rush. */
public final class Main {
    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        int checkpoint = Math.max(1, total / 4);
        Exporter exporter = new Exporter(new BufferPool());

        ArrayDeque<ExportJob> inFlight = new ArrayDeque<>();
        long checksum = 0;
        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            int pages = 1 + (i * 7) % 9;
            inFlight.addLast(exporter.start(new Document(i, pages)));
            int concurrency = Math.min(48, 4 + i / 100);
            while (inFlight.size() > concurrency) {
                checksum ^= exporter.complete(inFlight.pollFirst());
            }
            if (i % checkpoint == 0) {
                report(i, inFlight.size(), exporter);
            }
        }
        while (!inFlight.isEmpty()) {
            checksum ^= exporter.complete(inFlight.pollFirst());
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d jobs, %d pages, checksum %x, %d ms%n",
                exporter.completed(), exporter.pagesWritten(), checksum, millis);
    }

    private static void report(int handled, int inFlight, Exporter exporter) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long usedKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        System.out.printf("after %6d jobs: %3d in flight, pool owns %3d buffers (%3d idle), %6d unpooled, heap in use %7d KB%n",
                handled, inFlight, exporter.pool().owned(), exporter.pool().idle(), exporter.pool().unpooled(), usedKb);
    }
}
