package quotes;

import java.lang.management.ManagementFactory;
import java.lang.management.MemoryPoolMXBean;

/** Load driver: prices a stream of quote requests from a growing customer base. */
public final class Main {
    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        int checkpoint = Math.max(1, total / 4);
        QuoteService service = new QuoteService();

        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            // Every eighth request comes from a customer who asked a moment ago.
            long customer = (i % 8 == 0) ? i - 3 : i;
            service.quote(customer, 1 + (i % 40), 150 + (i * 37) % 9000);
            if (i % checkpoint == 0) {
                report(i);
            }
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d quotes, %d cents quoted, %d ms%n", service.quoted(), service.totalCents(), millis);
    }

    private static void report(int handled) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long heapKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        long metaspaceKb = 0;
        for (MemoryPoolMXBean pool : ManagementFactory.getMemoryPoolMXBeans()) {
            if (pool.getName().equals("Metaspace")) {
                metaspaceKb = pool.getUsage().getUsed() / 1024;
            }
        }
        int classes = ManagementFactory.getClassLoadingMXBean().getLoadedClassCount();
        System.out.printf("after %6d quotes: classes loaded %6d, metaspace %7d KB, heap in use %7d KB%n",
                handled, classes, metaspaceKb, heapKb);
    }
}
