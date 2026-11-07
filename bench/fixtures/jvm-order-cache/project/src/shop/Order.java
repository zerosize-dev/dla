package shop;

import java.util.List;

/** A priced order as confirmed to the customer. */
record Order(String id, String customer, List<Line> lines, long subtotalCents,
             long discountCents, String notes) {

    record Line(String sku, String title, int quantity, long unitCents) {
        long total() {
            return quantity * unitCents;
        }
    }

    long totalCents() {
        return subtotalCents - discountCents;
    }

    int itemCount() {
        int count = 0;
        for (Line line : lines) {
            count += line.quantity();
        }
        return count;
    }
}
