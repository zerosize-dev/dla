package audit;

/** Load driver: records administrative actions at a steady rate on a simulated clock. */
public final class Main {
    private static final String[] ACTORS = {"ines", "marcus", "chen", "odile", "tariq", "bao", "greta"};
    private static final String[] ACTIONS = {"plan.change", "seat.add", "seat.remove", "region.move", "contact.update", "limit.raise"};

    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 30000;
        int checkpoint = Math.max(1, total / 4);
        AuditService service = new AuditService();

        long clock = 1_700_000_000_000L;
        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            clock += 800 + (i * 37L) % 700;
            String action = ACTIONS[i % ACTIONS.length];
            service.record(clock, ACTORS[i % ACTORS.length], action, "account-" + (i % 5000), Payloads.describe(i, action));
            if (i % checkpoint == 0) {
                report(i, clock, service);
            }
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d actions recorded, %d shown, %d ms%n", service.recorded(), service.shown(), millis);
    }

    private static void report(int handled, long clock, AuditService service) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long usedKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        long hours = (clock - 1_700_000_000_000L) / 3_600_000L;
        System.out.printf("after %6d actions (%3d h of traffic): console lists %6d, heap in use %7d KB%n",
                handled, hours, service.shown(), usedKb);
    }
}
