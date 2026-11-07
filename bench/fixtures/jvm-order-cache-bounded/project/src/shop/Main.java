package shop;

/** Load driver: replays a stream of order requests against the service. */
public final class Main {
    private static final String[] CUSTOMERS = {"acme", "gold-harbor", "initech", "gold-vega", "umbrella", "globex"};

    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        int checkpoint = Math.max(1, total / 4);
        OrderService service = new OrderService(new PricingService());

        long revenue = 0;
        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            // Every eighth request is a client retrying one it sent a moment ago.
            int source = (i % 8 == 0) ? i - 3 : i;
            OrderRequest request = new OrderRequest(
                    CUSTOMERS[source % CUSTOMERS.length],
                    Integer.toHexString(source * 0x9E3779B1),
                    source,
                    describeDelivery(source));
            revenue += service.place(request).totalCents();
            if (i % checkpoint == 0) {
                report(i, service);
            }
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d requests, %d placed, %d replayed, revenue %d cents, %d ms%n",
                total, service.placed(), service.replayed(), revenue, millis);
    }

    private static String describeDelivery(int seed) {
        StringBuilder text = new StringBuilder("Deliver to dock ").append(seed % 40).append(", ring twice;");
        for (int i = 0; i < 6; i++) {
            text.append(" leave with reception desk ").append((seed + i) % 17).append(',');
        }
        return text.toString();
    }

    private static void report(int handled, OrderService service) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long usedKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        System.out.printf("after %6d requests: cached %6d orders, heap in use %7d KB%n",
                handled, OrderCache.size(), usedKb);
    }
}
