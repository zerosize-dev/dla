package @@DLA_PACKAGE@@;

import dla.harness.DlaDump;

/**
 * DLA loop driver. One process, one checkpoint record per round (x = units completed).
 * Fill the two DLA-HOOK bodies below and nothing else; keep the loop as it is.
 *
 * <p>Keys: DLA_KEY_MODE=fresh uses key = i (every unit sees a new key); DLA_KEY_MODE=repeated
 * uses key = i % repeat (the key space is bounded). Rounds: DLA_WARMUP warm-up rounds, then
 * DLA_ROUNDS measured rounds of DLA_STEP units each (the defaults are baked in at generation).
 */
public final class @@DLA_CLASS@@ {
    private @@DLA_CLASS@@() {}

    /** Builds the object graph under test once, before the first round. */
    static void setup() throws Exception {
        // DLA-HOOK setup: construct the services/caches/pools the workload needs (use package-private API if useful); keep references in static fields of this class if the unit needs them
        throw new UnsupportedOperationException("DLA-HOOK setup has not been filled in");
    }

    /**
     * One unit of work for the given key: the same call a request/job/event would make.
     * i is the 1-based unit number; key is already mapped for the key mode.
     */
    static void unit(int i, int key) throws Exception {
        // DLA-HOOK unit: perform exactly one unit of the workload for this key (derive request ids/names/payloads from key so a repeated key reproduces an earlier request)
        throw new UnsupportedOperationException("DLA-HOOK unit has not been filled in");
    }

    /**
     * Time limit of this process in seconds (`dla harness run` sets DLA_TIME_LIMIT_SECS from the
     * harness timeout; 0 or unset = no fitting). Once MIN_STEP units of a round have run (inside
     * the first round, so one slow round cannot outlast the limit before the first look) and after
     * every round, the loop lowers the step of the round in progress and of the rounds left when
     * the units left would not fit (margin: 15 percent and 5 seconds), never below MIN_STEP. 10
     * units: with fewer, the per-round costs (the forced collection and the checkpoint) and the
     * heap noise swamp a leak of one entry per unit. The x axis stays the cumulative unit count.
     */
    private static final double LIMIT_SECS = doubleEnv("DLA_TIME_LIMIT_SECS");
    private static final int MIN_STEP = 10;
    private static int step;
    private static int done;
    private static boolean warned;
    private static long loopStart;

    /** Lowers {@code step} when the units left would not fit; inRound units of the round have run. */
    private static void fit(int round, int total, int inRound) {
        if (!(LIMIT_SECS > 0) || done == 0) {
            return;
        }
        int left = total - round;
        double perUnit = (System.nanoTime() - loopStart) / 1e9 / done;
        double uptime = java.lang.management.ManagementFactory.getRuntimeMXBean().getUptime() / 1000.0;
        double budget = LIMIT_SECS * 0.85 - 5 - uptime;
        double need = perUnit * ((double) step - inRound + (double) step * left);
        if (need <= budget) {
            return;
        }
        int units = (int) (budget / perUnit);
        int lowered = Math.max(Math.min(MIN_STEP, step), Math.min(step, (units + inRound) / (left + 1)));
        if (lowered < step) {
            DlaDump.addLimitations(java.util.List.of(String.format(java.util.Locale.ROOT,
                "the driver lowered its step from %d to %d units per round in round %d (after %d units of it): one unit takes %.1f ms and the %s s time limit would cut the process (the x axis is still the cumulative unit count)",
                step, lowered, round, inRound, perUnit * 1000, Double.toString(LIMIT_SECS))));
            step = lowered;
        }
        need = perUnit * ((double) step - inRound + (double) step * left);
        if (!warned && need > budget) {
            warned = true;
            DlaDump.addLimitations(java.util.List.of(String.format(java.util.Locale.ROOT,
                "the unit is too slow for a series: even %d units per round (the floor) needs %.0f s for the units left and %.0f s remain; make the unit cheaper (one call, no sleeps, smaller payloads) or raise timeout_secs in harness.json",
                step, need, Math.max(0.0, budget))));
        }
    }

    /** Runs the rounds. */
    public static void main(String[] args) throws Exception {
        int code = 0;
        try {
            run();
        } catch (Throwable t) {
            t.printStackTrace();
            code = 1;
        }
        // The JVM ends when the loop is done and the last checkpoint is written: non-daemon threads
        // and servers of the units must not hold it until the kill (shutdown hooks still run).
        System.out.flush();
        System.err.flush();
        System.exit(code);
    }

    private static void run() throws Exception {
        int stepDefault = intEnv("DLA_STEP", @@DLA_STEP@@);
        int warmup = intEnv("DLA_WARMUP", @@DLA_WARMUP@@);
        int rounds = intEnv("DLA_ROUNDS", @@DLA_ROUNDS@@);
        int repeat = intEnv("DLA_REPEAT", @@DLA_REPEAT@@);
        boolean repeated = "repeated".equals(System.getenv("DLA_KEY_MODE"));

        setup();
        int total = warmup + rounds;
        step = stepDefault;
        done = 0;
        loopStart = System.nanoTime();
        for (int round = 1; round <= total; round++) {
            for (int k = 0; k < step; k++) {
                int i = ++done;
                unit(i, repeated ? i % repeat : i);
                if (k + 1 == Math.min(MIN_STEP, stepDefault)) {
                    fit(round, total, k + 1);
                }
            }
            DlaDump.checkpoint(done, round == total);
            if (round < total) {
                fit(round + 1, total, 0);
            }
        }
    }

    private static double doubleEnv(String name) {
        String v = System.getenv(name);
        try {
            return v == null || v.isEmpty() ? 0 : Double.parseDouble(v.trim());
        } catch (NumberFormatException e) {
            return 0;
        }
    }

    private static int intEnv(String name, int fallback) {
        String v = System.getenv(name);
        try {
            return v == null || v.isEmpty() ? fallback : Integer.parseInt(v.trim());
        } catch (NumberFormatException e) {
            return fallback;
        }
    }
}
