package audit;

/** Plain assertions; run with -ea. */
public final class AuditServiceTest {
    private static final long HOUR = 3_600_000L;

    public static void main(String[] args) {
        newestComesFirst();
        entriesOlderThanTheWindowDrop();
        actorFilter();
        payloadsAreStable();
        System.out.println("all tests passed");
    }

    private static void newestComesFirst() {
        AuditService service = new AuditService();
        service.record(1000, "ines", "plan.change", "account-1", "{}");
        service.record(2000, "chen", "seat.add", "account-2", "{}");
        service.record(3000, "ines", "seat.remove", "account-3", "{}");
        assert service.latest(2).get(0).target().equals("account-3");
        assert service.latest(2).get(1).target().equals("account-2");
        assert service.latest(10).size() == 3;
    }

    private static void entriesOlderThanTheWindowDrop() {
        AuditService service = new AuditService();
        service.record(0, "ines", "plan.change", "account-1", "{}");
        service.record(HOUR, "chen", "seat.add", "account-2", "{}");
        service.record(2 * HOUR, "chen", "seat.add", "account-3", "{}");
        assert service.shown() == 3;
        service.record(3 * HOUR + 1, "ines", "limit.raise", "account-4", "{}");
        assert service.shown() == 3 : service.shown();
        assert service.recorded() == 4;
        assert service.latest(10).stream().noneMatch(e -> e.target().equals("account-1"));
    }

    private static void actorFilter() {
        AuditService service = new AuditService();
        service.record(10, "ines", "plan.change", "account-1", "{}");
        service.record(20, "chen", "seat.add", "account-2", "{}");
        service.record(30, "ines", "seat.add", "account-3", "{}");
        assert service.activityOf("ines").size() == 2;
        assert service.activityOf("nobody").isEmpty();
    }

    private static void payloadsAreStable() {
        assert Payloads.describe(5, "seat.add").equals(Payloads.describe(5, "seat.add"));
        assert !Payloads.describe(5, "seat.add").equals(Payloads.describe(6, "seat.add"));
        assert Payloads.describe(5, "seat.add").contains("\"action\":\"seat.add\"");
    }
}
