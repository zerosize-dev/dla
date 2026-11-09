package ledger;

import reports.Event;
import reports.EventBus;
import reports.Extension;
import reports.Listener;

/** Keeps a running total of the sales published on the bus. */
public final class LedgerExtension implements Extension {
    /** Rounding table used when totals are converted for display. */
    private static final long[] ROUNDING = buildRounding();

    private final Tally tally = new Tally();

    @Override
    public String name() {
        return "ledger";
    }

    @Override
    public void start() {
        EventBus.subscribe(tally);
    }

    @Override
    public void stop() {
        EventBus.unsubscribe(tally);
    }

    @Override
    public long total() {
        return tally.total;
    }

    static long round(long cents) {
        return ROUNDING[(int) (cents % ROUNDING.length)];
    }

    private static long[] buildRounding() {
        long[] table = new long[1024];
        for (int i = 0; i < table.length; i++) {
            table[i] = i - i % 5;
        }
        return table;
    }

    private static final class Tally implements Listener {
        private long total;
        private long rounded;
        private int events;

        @Override
        public void onEvent(Event event) {
            events++;
            total += event.cents();
            rounded += round(event.cents());
        }
    }
}
