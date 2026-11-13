package audit;

/** One administrative action: who did what to which object, and the request payload. */
final class AuditEntry {
    private final long at;
    private final String actor;
    private final String action;
    private final String target;
    private final String payload;

    AuditEntry(long at, String actor, String action, String target, String payload) {
        this.at = at;
        this.actor = actor;
        this.action = action;
        this.target = target;
        this.payload = payload;
    }

    long at() {
        return at;
    }

    String actor() {
        return actor;
    }

    String action() {
        return action;
    }

    String target() {
        return target;
    }

    String payload() {
        return payload;
    }

    @Override
    public String toString() {
        return at + " " + actor + " " + action + " " + target;
    }
}
