package dla.harness;

import com.sun.management.HotSpotDiagnosticMXBean;
import java.lang.management.ManagementFactory;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import javax.management.MBeanServer;
import javax.management.ObjectName;

/**
 * Raw captures for the summarizers (DLA_CAPTURE): heap dumps, class histograms, a flight
 * recording with old-object samples, and the non-heap reports (thread dump, class loader
 * statistics, native memory). Files go to DLA_CAPTURE_DIR/&lt;pid&gt;/. Each capture is listed in
 * the "captures" array of the dump record that triggered it, with size and time. Nothing here
 * throws into the target: a failed capture becomes an entry with "error".
 *
 * <p>Environment: DLA_CAPTURE (comma list of heap, histogram, jfr, nonheap), DLA_CAPTURE_DIR,
 * DLA_CAPTURE_DUMPS (heap dumps in driver mode, default 4), and the driver's DLA_WARMUP and
 * DLA_ROUNDS, which place the dumps.
 */
public final class DlaCapture {
    private DlaCapture() {}

    // The flight recording is written by the JVM at exit; see start().
    private static final String RECORDING = "dla-capture";
    private static final int DEFAULT_DUMPS = 4;
    private static final Set<String> KINDS = new HashSet<>();
    private static int checkpoints;
    private static int heapSeq;
    private static int histSeq;
    private static boolean jfrStarted;
    private static boolean baselineTaken;
    private static boolean finalDone;
    private static Path dir;

    static {
        String v = DlaDump.env("DLA_CAPTURE");
        if (v != null) {
            for (String k : v.split(",")) {
                String t = k.trim().toLowerCase();
                if (!t.isEmpty()) {
                    KINDS.add(t);
                }
            }
        }
    }

    /** True when DLA_CAPTURE names at least one kind and DLA_CAPTURE_DIR is set. */
    public static boolean enabled() {
        return !KINDS.isEmpty() && DlaDump.env("DLA_CAPTURE_DIR") != null;
    }

    private static Path dir() throws Exception {
        if (dir == null) {
            Path d = Path.of(DlaDump.env("DLA_CAPTURE_DIR"), Long.toString(ProcessHandle.current().pid()));
            Files.createDirectories(d);
            dir = d;
        }
        return dir;
    }

    private static String diag(String op, String... args) throws Exception {
        MBeanServer s = ManagementFactory.getPlatformMBeanServer();
        ObjectName n = new ObjectName("com.sun.management:type=DiagnosticCommand");
        return (String) s.invoke(n, op, new Object[] {args}, new String[] {String[].class.getName()});
    }

    /**
     * Called once from premain: starts the flight recording before the workload runs (old-object
     * samples only cover objects allocated while it runs) and, for a command-mode process that
     * has no checkpoints, takes the native memory baseline.
     */
    public static synchronized void start(List<String> lim) {
        if (!enabled()) {
            return;
        }
        if (KINDS.contains("jfr")) {
            try {
                // The JVM writes the file in its own shutdown hook (dumponexit), after the old-object
                // path search; a dump from this agent's hook would race with the recording's
                // cleanup. The harness lists the file once the process has ended.
                String file = dir().resolve("recording.jfr").toString();
                // Keep the recording's chunks inside the capture directory: a killed process
                // would leave them in the temp directory, outside the run.
                try {
                    Path repo = dir().resolve("jfr-repo");
                    Files.createDirectories(repo);
                    diag("jfrConfigure", "repositorypath=" + repo);
                } catch (Throwable t) {
                    lim.add("flight recording repository could not be set: " + t);
                }
                String out = diag("jfrStart", "name=" + RECORDING, "settings=profile", "path-to-gc-roots=true",
                    "dumponexit=true", "filename=" + file);
                jfrStarted = true;
                if (out == null || out.isEmpty()) {
                    lim.add("jfrStart gave no output");
                }
            } catch (Throwable t) {
                lim.add("flight recording could not be started: " + t);
            }
        }
        if (KINDS.contains("nonheap") && DlaDump.env("DLA_ROUNDS") == null) {
            baseline(lim);
        }
    }

    private static void baseline(List<String> lim) {
        try {
            diag("vmNativeMemory", "baseline");
            baselineTaken = true;
        } catch (Throwable t) {
            lim.add("native memory baseline failed: " + t);
        }
    }

    private static int intEnv(String name, int fallback) {
        Long v = DlaDump.envLong(name);
        return v == null ? fallback : (int) Math.max(0, Math.min(v, Integer.MAX_VALUE));
    }

    /** Whether the checkpoint with this 1-based number gets a heap dump. */
    static boolean dumpHere(int seq, int warmup, int rounds, int dumps) {
        int total = warmup + rounds;
        if (total <= 0 || total <= dumps) {
            return true;
        }
        int first = Math.max(warmup, 1);
        if (seq == first || seq == total) {
            return true;
        }
        int interior = dumps - 2;
        if (interior <= 0 || seq < first || seq > total) {
            return false;
        }
        // Evenly spaced between the end of the warm-up and the last round.
        for (int j = 1; j <= interior; j++) {
            int at = first + Math.round((float) (total - first) * j / (interior + 1));
            if (seq == at) {
                return true;
            }
        }
        return false;
    }

    private static Map<String, Object> entry(String kind, String role, Path file, long ms, String error) {
        Map<String, Object> e = new LinkedHashMap<>();
        e.put("kind", kind);
        if (file != null) {
            e.put("file", file.getFileName().toString());
        }
        e.put("role", role);
        if (error != null) {
            e.put("error", error);
        } else {
            long size = 0;
            try {
                size = Files.size(file);
            } catch (Throwable ignored) {
                // reported as 0 bytes
            }
            e.put("bytes", size);
        }
        e.put("ms", ms);
        return e;
    }

    private static void text(List<Object> out, String kind, String role, String name, String op, String[] args) {
        long t0 = System.nanoTime();
        try {
            Path f = dir().resolve(name);
            Files.writeString(f, diag(op, args), StandardCharsets.UTF_8);
            out.add(entry(kind, role, f, (System.nanoTime() - t0) / 1_000_000, null));
        } catch (Throwable t) {
            out.add(entry(kind, role, null, (System.nanoTime() - t0) / 1_000_000, t.toString()));
        }
    }

    /**
     * Writes the captures due at this record and returns their descriptions. phase is
     * "checkpoint" or "exit"; last marks a driver's final round.
     */
    public static synchronized List<Object> capture(String phase, boolean last, List<String> lim) {
        List<Object> out = new ArrayList<>();
        if (!enabled()) {
            return out;
        }
        boolean driver = phase.equals("checkpoint");
        boolean fin = !driver || last;
        if (finalDone) {
            return out;
        }
        int seq = 0;
        int warmup = intEnv("DLA_WARMUP", 0);
        int rounds = intEnv("DLA_ROUNDS", 0);
        if (driver) {
            seq = ++checkpoints;
        }
        try {
            dir();
        } catch (Throwable t) {
            lim.add("capture directory unusable: " + t);
            return out;
        }
        if (KINDS.contains("histogram")) {
            histSeq++;
            text(out, "histogram", "class histogram", "histogram-" + histSeq + ".txt", "gcClassHistogram",
                new String[0]);
        }
        if (KINDS.contains("heap") && (!driver || dumpHere(seq, warmup, rounds, intEnv("DLA_CAPTURE_DUMPS", DEFAULT_DUMPS)))) {
            heapSeq++;
            long t0 = System.nanoTime();
            try {
                Path f = dir().resolve("heap-" + heapSeq + ".hprof");
                Files.deleteIfExists(f);
                ManagementFactory.getPlatformMXBean(HotSpotDiagnosticMXBean.class).dumpHeap(f.toString(), true);
                out.add(entry("heap", "heap dump (HPROF)", f, (System.nanoTime() - t0) / 1_000_000, null));
            } catch (Throwable t) {
                out.add(entry("heap", "heap dump (HPROF)", null, (System.nanoTime() - t0) / 1_000_000, t.toString()));
            }
        }
        if (driver && KINDS.contains("nonheap") && !baselineTaken && seq == Math.max(warmup, 1) && !last) {
            baseline(lim);
        }
        if (fin) {
            finalDone = true;
            finalCaptures(out, lim);
        }
        return out;
    }

    private static void finalCaptures(List<Object> out, List<String> lim) {
        if (KINDS.contains("nonheap")) {
            long t0 = System.nanoTime();
            try {
                Path f = dir().resolve("threads.json");
                Files.deleteIfExists(f);
                diag("threadDumpToFile", "-format=json", f.toString());
                out.add(entry("nonheap", "thread dump (json)", f, (System.nanoTime() - t0) / 1_000_000, null));
            } catch (Throwable t) {
                String why = t instanceof javax.management.JMException
                    ? "Thread.dump_to_file (threadDumpToFile) needs JDK 21 (verified; 19 and 20 untested): " + t
                    : t.toString();
                out.add(entry("nonheap", "thread dump (json)", null, (System.nanoTime() - t0) / 1_000_000, why));
            }
            text(out, "nonheap", "class loader statistics", "classloader-stats.txt", "vmClassloaderStats",
                new String[0]);
            text(out, "nonheap", "native memory summary", "nmt-summary.txt", "vmNativeMemory",
                new String[] {"summary"});
            if (baselineTaken) {
                text(out, "nonheap", "native memory change since baseline", "nmt-diff.txt", "vmNativeMemory",
                    new String[] {"summary.diff"});
            } else {
                lim.add("no native memory baseline: nmt-diff.txt was not written");
            }
        }
    }
}
