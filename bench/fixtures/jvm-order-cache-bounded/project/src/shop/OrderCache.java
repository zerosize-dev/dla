package shop;

import java.util.LinkedHashMap;
import java.util.Map;

/** Remembers confirmed orders so a client that repeats a request gets the same answer. */
final class OrderCache {
    private static final int CAPACITY = 1000;

    private static final Map<String, Order> ORDERS_BY_ID = new LinkedHashMap<>(256, 0.75f, true) {
        @Override
        protected boolean removeEldestEntry(Map.Entry<String, Order> eldest) {
            return size() > CAPACITY;
        }
    };

    private OrderCache() {}

    static synchronized Order find(String orderId) {
        return ORDERS_BY_ID.get(orderId);
    }

    static synchronized void store(Order order) {
        ORDERS_BY_ID.put(order.id(), order);
    }

    static synchronized int size() {
        return ORDERS_BY_ID.size();
    }

    /** Used by tests that need a clean slate. */
    static synchronized void invalidateAll() {
        ORDERS_BY_ID.clear();
    }
}
