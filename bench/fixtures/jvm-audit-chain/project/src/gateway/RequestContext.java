package gateway;

/** Per-request state: who called, what they asked for, and when it arrived. */
final class RequestContext {
    private static RequestContext latest;
    private static long counter;

    private final long sequence;
    private final String caller;
    private final String path;
    private final byte[] body;
    private final long receivedAtNanos;
    private final RequestContext previous;

    private RequestContext(long sequence, String caller, String path, byte[] body,
                           long receivedAtNanos, RequestContext previous) {
        this.sequence = sequence;
        this.caller = caller;
        this.path = path;
        this.body = body;
        this.receivedAtNanos = receivedAtNanos;
        this.previous = previous;
    }

    /** Starts the context for a newly arrived request. */
    static synchronized RequestContext open(String caller, String path, byte[] body, long nowNanos) {
        RequestContext context = new RequestContext(++counter, caller, path, body, nowNanos, latest);
        latest = context;
        return context;
    }

    long sequence() {
        return sequence;
    }

    String caller() {
        return caller;
    }

    String path() {
        return path;
    }

    byte[] body() {
        return body;
    }

    /** Microseconds between the previous request's arrival and this one's. */
    long microsSincePrevious() {
        return previous == null ? 0 : (receivedAtNanos - previous.receivedAtNanos) / 1000;
    }

    /** Path the previous request asked for, or "-" for the first request. */
    String previousPath() {
        return previous == null ? "-" : previous.path;
    }
}
