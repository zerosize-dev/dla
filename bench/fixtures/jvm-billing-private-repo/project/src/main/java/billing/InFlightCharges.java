package billing;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

/** Tracks charges that have been handed to the ledger and not yet confirmed. */
final class InFlightCharges {
    private static final Map<String, PendingCharge> AWAITING = new ConcurrentHashMap<>();

    private InFlightCharges() {}

    static void register(String ticketId, PendingCharge charge) {
        AWAITING.put(ticketId, charge);
    }

    static PendingCharge complete(String ticketId) {
        return AWAITING.remove(ticketId);
    }

    static int count() {
        return AWAITING.size();
    }

    /** Age of the longest-waiting charge, for the dashboard. */
    static long oldestAgeMillis(long nowNanos) {
        long oldest = 0;
        for (PendingCharge charge : AWAITING.values()) {
            oldest = Math.max(oldest, charge.ageMillis(nowNanos));
        }
        return oldest;
    }
}
