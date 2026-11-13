package audit;

import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;

/**
 * The most recent administrative actions, newest last. The operator console lists the last three
 * hours of activity from here; older history is read from the archive by the nightly job, so this
 * tail drops entries that are older than three hours when a new one arrives.
 */
final class AuditTail {
    static final long WINDOW_MILLIS = 3L * 60 * 60 * 1000;

    private final ArrayDeque<AuditEntry> entries = new ArrayDeque<>();

    synchronized void append(AuditEntry entry) {
        entries.addLast(entry);
        long cutoff = entry.at() - WINDOW_MILLIS;
        while (!entries.isEmpty() && entries.peekFirst().at() < cutoff) {
            entries.pollFirst();
        }
    }

    synchronized int size() {
        return entries.size();
    }

    /** The newest entries, newest first. */
    synchronized List<AuditEntry> newest(int limit) {
        List<AuditEntry> out = new ArrayList<>();
        Iterator<AuditEntry> it = entries.descendingIterator();
        while (it.hasNext() && out.size() < limit) {
            out.add(it.next());
        }
        return out;
    }

    synchronized List<AuditEntry> byActor(String actor) {
        List<AuditEntry> out = new ArrayList<>();
        for (AuditEntry entry : entries) {
            if (entry.actor().equals(actor)) {
                out.add(entry);
            }
        }
        return out;
    }
}
