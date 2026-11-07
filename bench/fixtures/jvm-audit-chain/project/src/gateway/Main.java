package gateway;

import java.nio.charset.StandardCharsets;

/** Load driver: sends a mixed stream of requests through the gateway. */
public final class Main {
    private static final int ACCOUNTS = 64;

    public static void main(String[] args) {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        int checkpoint = Math.max(1, total / 4);

        long[] auditLines = new long[1];
        long[] auditChars = new long[1];
        Gateway gateway = new Gateway(new Handlers(new Ledger(ACCOUNTS, 500_000)), line -> {
            auditLines[0]++;
            auditChars[0] += line.length();
        });

        int[] byStatus = new int[600];
        long clock = 0;
        long started = System.nanoTime();
        for (int i = 1; i <= total; i++) {
            clock += 1_000 + (i % 7) * 100;
            String path;
            String payload;
            switch (i % 5) {
                case 0, 1 -> {
                    path = "/balance";
                    payload = Ledger.account(i % ACCOUNTS);
                }
                case 2 -> {
                    path = "/statement";
                    payload = Ledger.account((i * 3) % ACCOUNTS);
                }
                case 3 -> {
                    path = "/transfer";
                    payload = Ledger.account(i % ACCOUNTS) + "," + Ledger.account((i * 5 + 1) % ACCOUNTS)
                            + "," + (100 + i % 900);
                }
                default -> {
                    path = i % 50 == 4 ? "/export" : "/balance";
                    payload = Ledger.account((i + 11) % ACCOUNTS);
                }
            }
            Response response = gateway.handle("client-" + (i % 12), path, pad(payload), clock);
            byStatus[response.status()]++;
            if (i % checkpoint == 0) {
                report(i, auditLines[0]);
            }
        }
        long millis = (System.nanoTime() - started) / 1_000_000;
        System.out.printf("done: %d requests (ok %d, rejected %d), %d audit chars, %d ms%n",
                gateway.served(), byStatus[200], gateway.served() - byStatus[200], auditChars[0], millis);
    }

    /** Clients send form-encoded bodies padded to a fixed envelope size. */
    private static byte[] pad(String payload) {
        StringBuilder body = new StringBuilder(payload);
        while (body.length() < 384) {
            body.append(' ');
        }
        return body.toString().getBytes(StandardCharsets.UTF_8);
    }

    private static void report(int handled, long auditLines) {
        System.gc();
        Runtime runtime = Runtime.getRuntime();
        long usedKb = (runtime.totalMemory() - runtime.freeMemory()) / 1024;
        System.out.printf("after %6d requests: %6d audit lines, heap in use %7d KB%n", handled, auditLines, usedKb);
    }
}
