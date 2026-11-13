package portal;

import java.util.ArrayDeque;

/** Load driver: replays logins, page requests, logouts and the occasional operator sign-out. */
public final class Main {
    private static final String[] PAGES = {"/home", "/inbox", "/reports", "/settings", "/billing"};

    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        int checkpoint = Math.max(1, total / 4);
        Directory directory = new Directory(300);
        SessionService service = new SessionService(directory);

        // About 200 people are signed in at any time: each new login pushes the oldest one out.
        ArrayDeque<Session> present = new ArrayDeque<>();
        long clock = 1_700_000_000_000L;
        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            clock += 40;
            Session session = service.login(directory.userAt(i), directory.deviceAt(i), clock);
            present.addLast(session);
            for (int p = 0; p < 3; p++) {
                service.request(session.id(), clock + p, PAGES[(i + p) % PAGES.length]);
            }
            if (present.size() > 200) {
                service.logout(present.pollFirst().id());
            }
            if (i % 500 == 0) {
                service.signOutEverywhere(directory.watchedUser(i / 500));
            }
            if (i % checkpoint == 0) {
                report(i, service);
            }
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d logins, %d logouts, %d ended by the operator, %d ms%n",
                service.signedIn(), service.signedOut(), service.removedByOperator(), millis);
    }

    private static void report(int handled, SessionService service) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long usedKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        System.out.printf("after %6d logins: %4d sessions open, heap in use %7d KB%n",
                handled, service.openSessions(), usedKb);
    }
}
