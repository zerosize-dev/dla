import dla.harness.DlaDump;
import java.io.PrintWriter;
import java.io.StringWriter;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import org.junit.platform.engine.TestExecutionResult;
import org.junit.platform.engine.UniqueId;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
import org.junit.platform.launcher.TestExecutionListener;
import org.junit.platform.launcher.TestIdentifier;
import org.junit.platform.launcher.core.LauncherDiscoveryRequestBuilder;
import org.junit.platform.launcher.core.LauncherFactory;
import org.junit.platform.launcher.listeners.SummaryGeneratingListener;
import org.junit.platform.launcher.listeners.TestExecutionSummary;

import static org.junit.platform.engine.discovery.DiscoverySelectors.selectClass;
import static org.junit.platform.engine.discovery.DiscoverySelectors.selectMethod;

/**
 * DLA JUnit repeat driver. One JVM, one JUnit Platform Launcher, one checkpoint record per round
 * (x = executions of the selected tests so far). There is nothing to fill in: it runs the tests the
 * project already has, again and again, and the agent measures the process between rounds.
 *
 * <p>A unit is one execution of the selected classes or methods ("pkg.Class" or
 * "pkg.Class#method"). Rounds: DLA_WARMUP warm-up rounds, then DLA_ROUNDS measured rounds of
 * DLA_STEP units each (the defaults are baked in at generation). A framework that caches state
 * between executions (a Spring test context) keeps it across units, so growth comes from the code
 * under test; a test that discards the cache (@DirtiesContext) rebuilds it every unit.
 *
 * <p>A unit whose tests fail is counted, and the first failure is printed in full. The exit code is
 * non-zero only when the launcher itself failed (3, before the first checkpoint), no test was found
 * (3, before the first checkpoint), no test passed or failed (3: every test was aborted by an
 * assumption, skipped, or a class failed before its tests ran), or every unit failed (2): a test
 * that fails on the first unit and on every later one may never have reached the code under test, so the run is
 * refused rather than reported as a flat series. A test that passes once and then fails because
 * state has built up did reach it, and its series is kept: the summary line counts the failed
 * units. After the last checkpoint the cause line (and the first failure) come before the last
 * line, the summary; the two early exits (launcher failure, no test found) print their cause line
 * last and no summary. A
 * parameterized test is selected as "pkg.Class#method(java.lang.String)".
 *
 * <p>Several classes run together (the test loop of the queue). After the first unit a class none of
 * whose tests passed (every test failed, or it failed before its tests ran: a service it needs is not
 * running) is dropped while another class passed: one class that cannot run must not sink the harness,
 * and the drop is printed and recorded as a limitation. A class that passes once and fails later is
 * kept. The loop also fits its rounds into the time limit of the process (DLA_TIME_LIMIT_SECS, set by
 * {@code dla harness run} from the harness timeout): once a unit has been timed the step is lowered
 * (never below one unit per round: the trend needs the points) and, when even that does not fit, the
 * slowest classes of the first unit are dropped (never the last one); both are recorded.
 */
public final class DlaJUnitLoop {
    private DlaJUnitLoop() {}

    private static final String[] SELECT = {@@DLA_SELECT@@};

    /** Tests that passed and failed per class, and the time of each class, in one unit. */
    private static final class Tally implements TestExecutionListener {
        final Map<String, int[]> counts = new LinkedHashMap<>();
        final Map<String, Long> nanos = new LinkedHashMap<>();
        private final Map<String, Long> started = new LinkedHashMap<>();
        final Map<String, String> firstFailure = new LinkedHashMap<>();

        /** The class an identifier belongs to, and whether the identifier is that class itself. */
        private static String[] classOf(TestIdentifier id) {
            List<UniqueId.Segment> segments = UniqueId.parse(id.getUniqueId()).getSegments();
            for (int i = 0; i < segments.size(); i++) {
                if (segments.get(i).getType().equals("class")) {
                    return new String[] {segments.get(i).getValue(), i == segments.size() - 1 ? "self" : ""};
                }
            }
            return null;
        }

        @Override
        public void executionStarted(TestIdentifier id) {
            String[] c = classOf(id);
            if (c != null && !c[1].isEmpty()) {
                started.put(c[0], System.nanoTime());
            }
        }

        @Override
        public void executionFinished(TestIdentifier id, TestExecutionResult result) {
            String[] c = classOf(id);
            if (c == null) {
                return;
            }
            int[] n = counts.computeIfAbsent(c[0], k -> new int[2]);
            boolean failed = result.getStatus() == TestExecutionResult.Status.FAILED;
            if (id.isTest()) {
                if (result.getStatus() == TestExecutionResult.Status.SUCCESSFUL) {
                    n[0]++;
                } else if (failed) {
                    n[1]++;
                }
            } else if (failed) {
                // The class itself failed (its context did not start): none of its tests ran.
                n[1]++;
            }
            if (failed) {
                firstFailure.putIfAbsent(c[0], result.getThrowable().map(Throwable::toString).orElse("failed"));
            }
            if (!c[1].isEmpty() && started.containsKey(c[0])) {
                nanos.merge(c[0], System.nanoTime() - started.get(c[0]), Long::sum);
            }
        }
    }

    private static final double LIMIT_SECS = doubleEnv("DLA_TIME_LIMIT_SECS");
    private static int step;
    private static int done;
    private static long unitNanos;
    private static long firstUnitNanos;
    private static List<String> active;
    private static Map<String, Long> classNanos = new LinkedHashMap<>();
    private static boolean warned;

    private static String classOfSelector(String s) {
        int i = s.indexOf('#');
        return i < 0 ? s : s.substring(0, i);
    }

    private static LauncherDiscoveryRequest requestOf(List<String> selectors) {
        LauncherDiscoveryRequestBuilder builder = LauncherDiscoveryRequestBuilder.request();
        for (String s : selectors) {
            builder.selectors(s.indexOf('#') < 0 ? selectClass(s) : selectMethod(s));
        }
        return builder.build();
    }

    private static double uptimeSecs() {
        return java.lang.management.ManagementFactory.getRuntimeMXBean().getUptime() / 1000.0;
    }

    private static Set<String> distinct(List<String> selectors) {
        Set<String> out = new LinkedHashSet<>();
        for (String s : selectors) {
            out.add(classOfSelector(s));
        }
        return out;
    }

    /**
     * Lowers the step, then drops the slowest classes of the first unit, when the units left would
     * not fit in the time limit (margin: 15 percent and 5 seconds). {@code inRound} units of the
     * round in progress have run, {@code left} rounds follow it. Returns true when classes were
     * dropped (the request is then rebuilt).
     */
    private static boolean fit(int round, int total, int inRound) {
        if (!(LIMIT_SECS > 0) || done == 0) {
            return false;
        }
        int left = total - round;
        double perUnit = (done >= 2 ? (double) (unitNanos - firstUnitNanos) / (done - 1) : firstUnitNanos) / 1e9;
        double budget = LIMIT_SECS * 0.85 - 5 - uptimeSecs();
        double need = perUnit * ((double) step - inRound + (double) step * left);
        if (need <= budget) {
            return false;
        }
        int units = (int) (budget / perUnit);
        int lowered = Math.max(1, Math.min(step, (units + inRound) / (left + 1)));
        if (lowered < step) {
            DlaDump.addLimitations(List.of(String.format(java.util.Locale.ROOT,
                "the test loop lowered its step from %d to %d runs of the tests per round in round %d: one run takes %.1f s and the %s s time limit would cut the process (the x axis is still the cumulative run count)",
                step, lowered, round, perUnit, Double.toString(LIMIT_SECS))));
            step = lowered;
        }
        need = perUnit * ((double) step - inRound + (double) step * left);
        boolean dropped = false;
        // Only a steady-state reading (a second run) decides to drop a class: the first run holds
        // the one-time costs (class loading, the Spring context).
        if (need > budget && done >= 2) {
            long all = 0;
            for (long n : classNanos.values()) {
                all += n;
            }
            while (need > budget && distinct(active).size() > 1 && all > 0) {
                String slowest = null;
                for (String c : distinct(active)) {
                    if (slowest == null || classNanos.getOrDefault(c, 0L) > classNanos.getOrDefault(slowest, 0L)) {
                        slowest = c;
                    }
                }
                long n = classNanos.getOrDefault(slowest, 0L);
                final String drop = slowest;
                active = new ArrayList<>(active);
                active.removeIf(x -> classOfSelector(x).equals(drop));
                need = need * (1.0 - (double) n / all);
                all -= n;
                dropped = true;
                String why = String.format(java.util.Locale.ROOT,
                    "dropped test class %s: it was %.1f s of the %.1f s a run of the tests takes; the runs left of the other classes need about %.0f s of the %.0f s the %s s time limit leaves",
                    drop, n / 1e9, perUnit, need, budget, Double.toString(LIMIT_SECS));
                DlaDump.addLimitations(List.of(why));
                System.err.println("DLA-JUNIT " + why);
            }
        }
        if (!warned && need > budget) {
            warned = true;
            DlaDump.addLimitations(List.of(String.format(java.util.Locale.ROOT,
                "the tests are too slow for a series: even one run per round needs %.0f s for the runs left and %.0f s remain; choose fewer or faster test classes (--tests) or raise timeout_secs in harness.json",
                need, Math.max(0.0, budget))));
        }
        return dropped;
    }

    /**
     * After the first unit: a class none of whose tests passed (every test failed, or the class
     * failed before its tests ran) is dropped while another class had a test that passed. Returns
     * true when a class was dropped.
     */
    private static boolean dropClassesThatCannotRun(Tally tally) {
        Set<String> classes = distinct(active);
        if (classes.size() < 2) {
            return false;
        }
        boolean anyPassed = false;
        List<String> broken = new ArrayList<>();
        for (String c : classes) {
            int[] n = tally.counts.getOrDefault(c, new int[2]);
            if (n[0] > 0) {
                anyPassed = true;
            } else if (n[1] > 0) {
                broken.add(c);
            }
        }
        if (!anyPassed) {
            return false;
        }
        for (String c : broken) {
            String why = "dropped test class " + c + ": none of its tests passed in the first run ("
                + tally.firstFailure.getOrDefault(c, "failed")
                + "), so it is left out of the loop and the other classes are measured";
            DlaDump.addLimitations(List.of(why));
            System.err.println("DLA-JUNIT " + why);
            active = new ArrayList<>(active);
            active.removeIf(x -> classOfSelector(x).equals(c));
        }
        return !broken.isEmpty();
    }

    public static void main(String[] args) throws Exception {
        int stepDefault = intEnv("DLA_STEP", @@DLA_STEP@@);
        int warmup = intEnv("DLA_WARMUP", @@DLA_WARMUP@@);
        int rounds = intEnv("DLA_ROUNDS", @@DLA_ROUNDS@@);

        active = new ArrayList<>(List.of(SELECT));
        LauncherDiscoveryRequest request = requestOf(active);
        Launcher launcher = LauncherFactory.create();

        int total = warmup + rounds;
        step = stepDefault;
        done = 0;
        int failedUnits = 0;
        long testsRun = 0;
        long abortedTests = 0;
        long skippedTests = 0;
        String firstFailure = null;
        for (int round = 1; round <= total; round++) {
            for (int k = 0; k < step; k++) {
                SummaryGeneratingListener listener = new SummaryGeneratingListener();
                Tally tally = new Tally();
                long t0 = System.nanoTime();
                try {
                    launcher.execute(request, listener, tally);
                } catch (RuntimeException e) {
                    // The launcher itself failed (a selected class does not exist, no engine).
                    System.err.println("DLA-JUNIT cannot run " + String.join(", ", SELECT) + ": "
                            + chain(e));
                    System.err.flush();
                    System.exit(3);
                }
                long took = System.nanoTime() - t0;
                TestExecutionSummary summary = listener.getSummary();
                done++;
                unitNanos += took;
                if (done == 1) {
                    firstUnitNanos = took;
                    classNanos = new LinkedHashMap<>(tally.nanos);
                }
                if (done == 1 && summary.getTestsFoundCount() == 0) {
                    System.err.println("DLA-JUNIT no test found for " + String.join(", ", SELECT)
                            + ": check the class names (--tests) and that the class is a JUnit 5 test");
                    System.err.flush();
                    System.exit(3);
                }
                // A test aborted by an assumption (assumeTrue on a missing service) never reached
                // the code under test: it is not a test that ran.
                // A class whose @BeforeAll assumption failed is an aborted container.
                long aborted = summary.getTestsAbortedCount();
                abortedTests += aborted + summary.getContainersAbortedCount();
                // The listener already counts the tests of a skipped (@Disabled) class as skipped tests.
                skippedTests += summary.getTestsSkippedCount();
                testsRun += summary.getTestsStartedCount() - aborted;
                if (summary.getTotalFailureCount() > 0) {
                    failedUnits++;
                    if (firstFailure == null) {
                        firstFailure = describe(summary);
                        StringWriter w = new StringWriter();
                        summary.printFailuresTo(new PrintWriter(w), 25);
                        System.err.println("DLA-JUNIT first failing unit: " + done);
                        System.err.print(w);
                    }
                }
                boolean changed = false;
                if (done == 1) {
                    changed = dropClassesThatCannotRun(tally);
                }
                if (done <= 2) {
                    changed |= fit(round, total, k + 1);
                }
                if (changed) {
                    request = requestOf(active);
                }
            }
            DlaDump.checkpoint(done, round == total);
            if (round < total && fit(round + 1, total, 0)) {
                request = requestOf(active);
            }
        }
        int code = testsRun == 0 ? 3 : failedUnits == done ? 2 : 0;
        // The cause first, the summary line last: the log ends with the counts.
        if (testsRun == 0 && firstFailure != null) {
            System.err.println("DLA-JUNIT no test passed or failed: a class or test failed before any test ran: "
                    + firstFailure);
        } else if (testsRun == 0) {
            System.err.println("DLA-JUNIT no test passed or failed (aborted=" + abortedTests + " by an assumption,"
                    + " skipped=" + skippedTests + " disabled or conditionally off): e.g. a service the test"
                    + " needs is not running, or the tests are disabled here");
        }
        if (code == 2) {
            System.err.println("DLA-JUNIT every unit failed: " + firstFailure);
            if (firstFailure != null && firstFailure.contains("self-attach")) {
                System.err.println("DLA-JUNIT the mocking library could not attach to the running JVM (a hardened"
                        + " container or sandbox refuses it). Load it when the JVM starts: -javaagent:<the"
                        + " mockito-core jar, or the byte-buddy-agent jar when mockito-core has no Premain-Class>"
                        + " in the run arguments (dla-junit-build.sh adds it by itself when such a jar is on the"
                        + " test class path), or run where the attach is allowed");
            }
        }
        System.err.println("DLA-JUNIT summary: units=" + done + " tests_run=" + testsRun
                + " aborted=" + abortedTests
                + " skipped=" + skippedTests
                + " failed_units=" + failedUnits
                + (firstFailure == null ? "" : " first_failure=" + firstFailure));
        System.err.flush();
        // Not a plain return: an application started by the tests may leave non-daemon threads.
        System.exit(code);
    }

    private static String describe(TestExecutionSummary summary) {
        TestExecutionSummary.Failure f = summary.getFailures().get(0);
        Throwable t = f.getException();
        Throwable root = t;
        while (root.getCause() != null && root.getCause() != root) {
            root = root.getCause();
        }
        String text = t.getClass().getName() + ": " + oneLine(t.getMessage());
        if (root != t) {
            text += " (root cause " + root.getClass().getName() + ": " + oneLine(root.getMessage()) + ")";
        }
        return text;
    }

    /** An exception with its causes, outermost first, down to the root. */
    private static String chain(Throwable t) {
        StringBuilder b = new StringBuilder();
        for (int depth = 0; t != null && depth < 6; depth++) {
            if (depth > 0) {
                b.append(" <- caused by ");
            }
            b.append(t.getClass().getName()).append(": ").append(oneLine(t.getMessage()));
            t = t.getCause() == t ? null : t.getCause();
        }
        return b.toString();
    }

    private static String oneLine(String s) {
        if (s == null) {
            return "";
        }
        String t = s.replace('\r', ' ').replace('\n', ' ').trim();
        return t.length() > 300 ? t.substring(0, 300) + "..." : t;
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
