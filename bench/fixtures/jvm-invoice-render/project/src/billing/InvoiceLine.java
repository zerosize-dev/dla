package billing;

/** One priced line of an invoice. */
record InvoiceLine(String sku, String description, int quantity, long unitCents) {
    long totalCents() {
        return quantity * unitCents;
    }
}
