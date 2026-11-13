package audit;

/** Records administrative actions and answers the console's questions about them. */
final class AuditService {
    private final AuditTail tail = new AuditTail();
    private long recorded;

    void record(long at, String actor, String action, String target, String payload) {
        tail.append(new AuditEntry(at, actor, action, target, payload));
        recorded++;
    }

    long recorded() {
        return recorded;
    }

    int shown() {
        return tail.size();
    }

    java.util.List<AuditEntry> latest(int limit) {
        return tail.newest(limit);
    }

    java.util.List<AuditEntry> activityOf(String actor) {
        return tail.byActor(actor);
    }
}
