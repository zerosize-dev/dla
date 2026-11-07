package billing;

import java.util.concurrent.atomic.AtomicLong;

/** Sends (here: counts) the messages customers receive about their payments. */
public class CustomerNotifier {
    private final AtomicLong paid = new AtomicLong();
    private final AtomicLong declined = new AtomicLong();
    private final AtomicLong delayed = new AtomicLong();

    void paid(String email, String invoiceNumber) {
        paid.incrementAndGet();
    }

    void declined(String email, String invoiceNumber, String reasonCode) {
        declined.incrementAndGet();
    }

    void delayed(String email, String invoiceNumber) {
        delayed.incrementAndGet();
    }

    public String summary() {
        return "paid=" + paid.get() + " declined=" + declined.get() + " delayed=" + delayed.get();
    }

    long paidCount() {
        return paid.get();
    }
}
