package billing;

import java.util.List;

/** Renders invoices for the request loop that calls it. */
final class InvoiceService {
    private final InvoiceRenderer renderer = new InvoiceRenderer();

    String renderNow(Invoice invoice) {
        return renderer.render(invoice);
    }

    /** Renders a batch and returns the total number of characters produced. */
    long renderBatch(List<Invoice> batch) {
        long chars = 0;
        for (Invoice invoice : batch) {
            chars += renderer.render(invoice).length();
        }
        return chars;
    }
}
