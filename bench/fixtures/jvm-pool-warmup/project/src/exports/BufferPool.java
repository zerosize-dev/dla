package exports;

import java.util.ArrayDeque;

/**
 * Reusable page buffers, so a busy exporter does not allocate a fresh array for every page.
 * The pool owns at most {@link #SIZE} buffers; when they are all out, a caller gets a plain
 * array that the pool does not take back.
 */
final class BufferPool {
    static final int BUFFER_SIZE = 16 * 1024;
    static final int SIZE = 200;

    /** A buffer on loan; pooled leases return to the pool, the others are dropped. */
    static final class Lease {
        private final byte[] bytes = new byte[BUFFER_SIZE];
        private final boolean pooled;

        private Lease(boolean pooled) {
            this.pooled = pooled;
        }

        byte[] bytes() {
            return bytes;
        }
    }

    private final ArrayDeque<Lease> idle = new ArrayDeque<>();
    private int owned;
    private int unpooled;

    synchronized Lease acquire() {
        Lease lease = idle.pollLast();
        if (lease != null) {
            return lease;
        }
        if (owned < SIZE) {
            owned++;
            return new Lease(true);
        }
        unpooled++;
        return new Lease(false);
    }

    synchronized void release(Lease lease) {
        if (lease.pooled) {
            idle.addLast(lease);
        }
    }

    synchronized int idle() {
        return idle.size();
    }

    synchronized int owned() {
        return owned;
    }

    synchronized int unpooled() {
        return unpooled;
    }
}
