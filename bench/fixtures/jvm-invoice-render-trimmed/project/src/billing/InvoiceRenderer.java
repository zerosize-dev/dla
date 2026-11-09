package billing;

import java.util.Locale;

/** Turns an invoice into printable text. */
final class InvoiceRenderer {
    String render(Invoice invoice) {
        int mark = RenderContext.mark();
        RenderContext.append(String.format("INVOICE %d", invoice.number()));
        RenderContext.append("Customer: " + invoice.customer());
        long tax = 0;
        for (InvoiceLine line : invoice.lines()) {
            RenderContext.append(formatLine(line));
            tax += TaxTable.taxCents(line);
        }
        RenderContext.append("Subtotal: " + money(invoice.subtotalCents()));
        RenderContext.append("Tax: " + money(tax));
        RenderContext.append("Total: " + money(invoice.subtotalCents() + tax));
        String text = RenderContext.textFrom(mark);
        RenderContext.truncate(mark);
        return text;
    }

    private static String formatLine(InvoiceLine line) {
        return String.format(Locale.ROOT, "%-8s %-40s %4d x %10s = %10s",
                line.sku(), line.description(), line.quantity(),
                money(line.unitCents()), money(line.totalCents()));
    }

    static String money(long cents) {
        return String.format(Locale.ROOT, "%d.%02d", cents / 100, cents % 100);
    }
}
