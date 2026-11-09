package billing;

import java.util.Map;

/** Tax rates in basis points by SKU prefix. */
final class TaxTable {
    private static final Map<String, Integer> RATES = Map.of(
            "bk", 0,
            "fd", 500,
            "el", 2000,
            "ap", 1900);

    private TaxTable() {}

    static long taxCents(InvoiceLine line) {
        String prefix = line.sku().length() >= 2 ? line.sku().substring(0, 2) : line.sku();
        int rate = RATES.getOrDefault(prefix, 2000);
        return line.totalCents() * rate / 10_000;
    }
}
