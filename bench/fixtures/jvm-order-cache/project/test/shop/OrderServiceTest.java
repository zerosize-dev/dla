package shop;

/** Plain assertions; run with -ea. */
public final class OrderServiceTest {
    public static void main(String[] args) {
        discountsApplyAtThreshold();
        repeatedRequestReturnsSameOrder();
        distinctRequestsGetDistinctOrders();
        System.out.println("all tests passed");
    }

    private static OrderRequest request(String customer, String id, int seed) {
        return new OrderRequest(customer, id, seed, "leave at door");
    }

    private static void discountsApplyAtThreshold() {
        PricingService pricing = new PricingService();
        Order.Line big = new Order.Line("SKU-105", "Whiteboard", 2, 8990);
        Order.Line small = new Order.Line("SKU-108", "Sticky notes", 1, 380);
        Order large = pricing.price("a", "acme", java.util.List.of(big), "");
        Order tiny = pricing.price("b", "acme", java.util.List.of(small), "");
        Order loyal = pricing.price("c", "gold-vega", java.util.List.of(big), "");
        assert large.discountCents() == 17980 * 5 / 100 : large.discountCents();
        assert tiny.discountCents() == 0 : tiny.discountCents();
        assert loyal.discountCents() == 17980 * 8 / 100 : loyal.discountCents();
        assert large.totalCents() == 17980 - 899 : large.totalCents();
    }

    private static void repeatedRequestReturnsSameOrder() {
        OrderCache.invalidateAll();
        OrderService service = new OrderService(new PricingService());
        Order first = service.place(request("acme", "r1", 3));
        Order again = service.place(request("acme", "r1", 3));
        assert first == again;
        assert service.placed() == 1 && service.replayed() == 1;
    }

    private static void distinctRequestsGetDistinctOrders() {
        OrderCache.invalidateAll();
        OrderService service = new OrderService(new PricingService());
        Order a = service.place(request("acme", "r1", 3));
        Order b = service.place(request("acme", "r2", 4));
        assert !a.id().equals(b.id());
        assert OrderCache.size() == 2;
        assert a.itemCount() > 0;
    }
}
