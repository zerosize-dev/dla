package reports;

/** A completed sale, published once per processed order. */
public record Event(long orderId, String region, long cents) {}
