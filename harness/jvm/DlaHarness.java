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

    /** Runs the rounds. */
    public static void main(String[] args) throws Exception {
        int step = intEnv("DLA_STEP", @@DLA_STEP@@);
        int warmup = intEnv("DLA_WARMUP", @@DLA_WARMUP@@);
        int rounds = intEnv("DLA_ROUNDS", @@DLA_ROUNDS@@);
        int repeat = intEnv("DLA_REPEAT", @@DLA_REPEAT@@);
        boolean repeated = "repeated".equals(System.getenv("DLA_KEY_MODE"));

        setup();
        int total = warmup + rounds;
        int done = 0;
        for (int round = 1; round <= total; round++) {
            for (int k = 0; k < step; k++) {
                int i = ++done;
                unit(i, repeated ? i % repeat : i);
            }
            DlaDump.checkpoint(done, round == total);
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
