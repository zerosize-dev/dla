package billing;

import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/** Deterministic stream of invoices for the load driver. */
final class InvoiceFeed {
    private static final String[] SKUS = {"bk-1001", "fd-2040", "el-3300", "ap-4120", "el-3310"};
    private static final String[] NAMES = {
        "Hardcover notebook, ruled", "Roasted almonds, salted", "USB-C cable, braided",
        "Cotton work shirt", "Wireless mouse, compact"
    };

    private final Random random;
    private long next = 1;

    InvoiceFeed(long seed) {
        this.random = new Random(seed);
    }

    Invoice nextInvoice() {
        int count = 3 + random.nextInt(10);
        List<InvoiceLine> lines = new ArrayList<>(count);
        for (int i = 0; i < count; i++) {
            int k = random.nextInt(SKUS.length);
            lines.add(new InvoiceLine(SKUS[k], NAMES[k] + " #" + random.nextInt(900), 1 + random.nextInt(6),
                    199 + random.nextInt(9000)));
        }
        return new Invoice(next++, "customer-" + random.nextInt(5000), lines);
    }

    List<Invoice> nextBatch(int size) {
        List<Invoice> batch = new ArrayList<>(size);
        for (int i = 0; i < size; i++) {
            batch.add(nextInvoice());
        }
        return batch;
    }
}
