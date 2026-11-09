package billing;

import java.util.List;

/** An invoice for one customer, as received from the order system. */
record Invoice(long number, String customer, List<InvoiceLine> lines) {
    long subtotalCents() {
        long sum = 0;
        for (InvoiceLine line : lines) {
            sum += line.totalCents();
        }
        return sum;
    }
}
