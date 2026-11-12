import dla.harness.DlaDump;
import java.io.PrintWriter;
import java.io.StringWriter;
import org.junit.platform.launcher.Launcher;
import org.junit.platform.launcher.LauncherDiscoveryRequest;
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
 */
public final class DlaJUnitLoop {
    private DlaJUnitLoop() {}

    private static final String[] SELECT = {@@DLA_SELECT@@};

    public static void main(String[] args) throws Exception {
        int step = intEnv("DLA_STEP", @@DLA_STEP@@);
        int warmup = intEnv("DLA_WARMUP", @@DLA_WARMUP@@);
        int rounds = intEnv("DLA_ROUNDS", @@DLA_ROUNDS@@);

        LauncherDiscoveryRequestBuilder builder = LauncherDiscoveryRequestBuilder.request();
        for (String s : SELECT) {
            builder.selectors(s.indexOf('#') < 0 ? selectClass(s) : selectMethod(s));
        }
        LauncherDiscoveryRequest request = builder.build();
        Launcher launcher = LauncherFactory.create();

        int total = warmup + rounds;
        int done = 0;
        int failedUnits = 0;
        long testsRun = 0;
        long abortedTests = 0;
        long skippedTests = 0;
        String firstFailure = null;
        for (int round = 1; round <= total; round++) {
            for (int k = 0; k < step; k++) {
                SummaryGeneratingListener listener = new SummaryGeneratingListener();
                try {
                    launcher.execute(request, listener);
                } catch (RuntimeException e) {
                    // The launcher itself failed (a selected class does not exist, no engine).
                    System.err.println("DLA-JUNIT cannot run " + String.join(", ", SELECT) + ": "
                            + chain(e));
                    System.err.flush();
                    System.exit(3);
                }
                TestExecutionSummary summary = listener.getSummary();
                done++;
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
            }
            DlaDump.checkpoint(done, round == total);
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

    private static int intEnv(String name, int fallback) {
        String v = System.getenv(name);
        try {
            return v == null || v.isEmpty() ? fallback : Integer.parseInt(v.trim());
        } catch (NumberFormatException e) {
            return fallback;
        }
    }
}
