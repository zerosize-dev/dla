package shop;

import java.util.List;

/** Applies volume and loyalty discounts to a basket. */
final class PricingService {
    private static final long VOLUME_THRESHOLD_CENTS = 10_000;

    Order price(String id, String customer, List<Order.Line> lines, String notes) {
        long subtotal = 0;
        for (Order.Line line : lines) {
            subtotal += line.total();
        }
        long discount = 0;
        if (subtotal >= VOLUME_THRESHOLD_CENTS) {
            discount += subtotal * 5 / 100;
        }
        if (customer.startsWith("gold-")) {
            discount += subtotal * 3 / 100;
        }
        return new Order(id, customer, List.copyOf(lines), subtotal, discount, notes);
    }
}
