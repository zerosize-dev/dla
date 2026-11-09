package billing;

import java.util.List;

public final class InvoiceServiceTest {
    public static void main(String[] args) throws Exception {
        rendersTotals();
        taxByPrefix();
        batchRendersEveryInvoice();
        System.out.println("all tests passed");
    }

    private static void rendersTotals() {
        Invoice inv = new Invoice(1, "c-1", List.of(new InvoiceLine("el-1", "cable", 2, 1000)));
        String text = new InvoiceRenderer().render(inv);
        check(text.contains("Subtotal: 20.00"), "subtotal");
        check(text.contains("Tax: 4.00"), "tax");
        check(text.contains("Total: 24.00"), "total");
    }

    private static void taxByPrefix() {
        check(TaxTable.taxCents(new InvoiceLine("bk-1", "book", 1, 1000)) == 0, "books untaxed");
        check(TaxTable.taxCents(new InvoiceLine("fd-1", "food", 1, 1000)) == 50, "food rate");
    }

    private static void batchRendersEveryInvoice() throws Exception {
        long chars = new InvoiceService().renderBatch(new InvoiceFeed(1).nextBatch(20));
        check(chars > 0, "batch output");
    }

    private static void check(boolean ok, String what) {
        if (!ok) {
            throw new AssertionError(what);
        }
    }
}
