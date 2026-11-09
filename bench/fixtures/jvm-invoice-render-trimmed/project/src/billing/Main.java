package billing;

import java.util.List;

/** Load driver: renders the given number of invoices in batches and reports heap use. */
public final class Main {
    private static final int BATCH = 200;

    public static void main(String[] args) throws Exception {
        long total = args.length > 0 ? Long.parseLong(args[0]) : 5000;
        InvoiceFeed feed = new InvoiceFeed(7);
        long chars = 0;
        long done = 0;
        long nextReport = total / 4;
        InvoiceService service = new InvoiceService();
        while (done < total) {
            int n = (int) Math.min(BATCH, total - done);
            List<Invoice> batch = feed.nextBatch(n);
            chars += service.renderBatch(batch);
            done += n;
            if (done >= nextReport || done == total) {
                System.out.printf("invoices=%d chars=%d heapKB=%d%n", done, chars, usedKb());
                nextReport += Math.max(1, total / 4);
            }
        }
        System.exit(0);
    }

    private static long usedKb() {
        Runtime rt = Runtime.getRuntime();
        System.gc();
        return (rt.totalMemory() - rt.freeMemory()) / 1024;
    }
}
