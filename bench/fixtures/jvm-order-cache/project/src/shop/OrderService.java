package shop;

import java.util.ArrayList;
import java.util.List;

/** Places orders: prices the basket and answers repeats from the cache. */
final class OrderService {
    private final PricingService pricing;
    private int placed;
    private int replayed;

    OrderService(PricingService pricing) {
        this.pricing = pricing;
    }

    Order place(OrderRequest request) {
        String id = request.orderId();
        Order known = OrderCache.find(id);
        if (known != null) {
            replayed++;
            return known;
        }
        Order order = pricing.price(id, request.customer(), basket(request.basketSeed()), request.notes());
        OrderCache.store(order);
        placed++;
        return order;
    }

    private List<Order.Line> basket(int seed) {
        List<Order.Line> lines = new ArrayList<>();
        int distinct = 2 + Math.floorMod(seed, 4);
        for (int i = 0; i < distinct; i++) {
            Catalog.Product product = Catalog.at(seed * 7 + i * 3);
            int quantity = 1 + Math.floorMod(seed + i, 5);
            lines.add(new Order.Line(product.sku(), product.title(), quantity, product.priceCents()));
        }
        return lines;
    }

    int placed() {
        return placed;
    }

    int replayed() {
        return replayed;
    }
}
