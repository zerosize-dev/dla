package dla.harness;

import java.io.IOException;
import java.lang.instrument.Instrumentation;
import java.lang.management.ClassLoadingMXBean;
import java.lang.management.ManagementFactory;
import java.lang.management.MemoryMXBean;
import java.lang.ref.Reference;
import java.lang.reflect.AccessibleObject;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.HashMap;
import java.util.IdentityHashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;
import java.util.regex.Matcher;
import java.util.regex.Pattern;
import javax.management.MBeanServer;
import javax.management.ObjectName;

/**
 * Dump and probe code shared by the premain agent (DlaAgent) and the driver template. It writes
 * one JSON record per call to the file named by DLA_HARNESS_OUT. No dependencies; every public
 * entry point catches Throwable.
 *
 * <p>Environment: DLA_HARNESS_OUT, DLA_HARNESS_PROBES (probes.json), DLA_PURGE, DLA_PROJECT,
 * DLA_TICK_MS, DLA_ITER.
 */
public final class DlaDump {
    /** Upper bound of objects visited by one reachable-object walk. */
    public static final int MAX_OBJECTS = 2_000_000;
    private static final int TYPES_TOP = 20;
    private static final int COVERAGE_CAP = 4000;
    private static final Object PRESENT = new Object();

    private static volatile Instrumentation inst;
    private static volatile boolean checkpointWritten;

    private DlaDump() {}

    // ------------------------------------------------------------------ environment

    /** Returns the environment variable or null when unset or empty. */
    public static String env(String name) {
        String v = System.getenv(name);
        return v == null || v.isEmpty() ? null : v;
    }

    /** Returns the environment variable parsed as a long, or null. */
    public static Long envLong(String name) {
        try {
            String v = env(name);
            return v == null ? null : Long.valueOf(v.trim());
        } catch (NumberFormatException e) {
            return null;
        }
    }

    private static final List<String> EARLY = new ArrayList<>();

    /** Notes made before the first record (premain); they join the first record's limitations. */
    public static synchronized void addLimitations(List<String> notes) {
        EARLY.addAll(notes);
    }

    /** Remembers the instrumentation handle (used for loaded-class coverage and class lookup). */
    public static void setInstrumentation(Instrumentation i) {
        inst = i;
    }

    /** True once a checkpoint record was written in this JVM (a driver is running). */
    public static boolean checkpointWritten() {
        return checkpointWritten;
    }

    /** True for build tool JVMs (javac, jar, maven, gradle, ...), which are never measured. */
    public static boolean isToolJvm() {
        try {
            String cmd = System.getProperty("sun.java.command", "");
            String head = cmd.split("\\s+", 2)[0];
            if (cmd.startsWith("-m ") || cmd.startsWith("--module ")) {
                head = cmd;
            }
            String[] tools = {"com.sun.tools.", "sun.tools.", "jdk.compiler", "jdk.jartool", "jdk.jshell",
                "jdk.jdeps", "jdk.javadoc", "jdk.jcmd", "jdk.jlink", "org.apache.maven", "org.codehaus.plexus",
                "plexus-classworlds", "org.gradle", "GradleWrapperMain", "gradle-launcher", "gradle-wrapper",
                "org.jetbrains.kotlin", "org.apache.tools.ant", "sbt-launch"};
            for (String t : tools) {
                if (head.contains(t)) {
                    return true;
                }
            }
            return head.endsWith("/javac") || head.endsWith("/jar") || head.equals("javac") || head.equals("jar");
        } catch (Throwable t) {
            return false;
        }
    }

    // ------------------------------------------------------------------ public dump API

    /**
     * Writes a checkpoint record for the driver: x is the number of units completed; last marks
     * the final round, which also applies DLA_PURGE.
     */
    public static void checkpoint(long x, boolean last) {
        checkpointWritten = true;
        dump("checkpoint", x, last);
    }

    /** Writes one record. "tick" records are light (no forced GC, no probes). Never throws. */
    public static synchronized void dump(String phase, Long x, boolean last) {
        String out = env("DLA_HARNESS_OUT");
        if (out == null) {
            return;
        }
        List<String> limitations = new ArrayList<>(EARLY);
        // Premain notes (a failed flight recording) must reach a record the harness reads:
        // ticks are not read, so only a checkpoint or exit record takes them over.
        if (!phase.equals("tick")) {
            EARLY.clear();
        }
        Map<String, Object> rec = new LinkedHashMap<>();
        try {
            build(rec, limitations, phase, x, last);
        } catch (Throwable t) {
            limitations.add("dump failed: " + t);
        }
        try {
            rec.putIfAbsent("dla_harness", 1);
            rec.putIfAbsent("phase", phase);
            rec.put("limitations", limitations);
            Path p = Path.of(out);
            if (p.getParent() != null) {
                Files.createDirectories(p.getParent());
            }
            Files.writeString(p, Json.write(rec) + "\n", StandardCharsets.UTF_8, StandardOpenOption.CREATE,
                StandardOpenOption.APPEND);
        } catch (Throwable ignored) {
            // nothing more can be done
        }
    }

    private static void build(Map<String, Object> rec, List<String> lim, String phase, Long x, boolean last) {
        boolean tick = phase.equals("tick");
        rec.put("dla_harness", 1);
        rec.put("phase", phase);
        rec.put("pid", ProcessHandle.current().pid());
        String cmd = System.getProperty("sun.java.command", "");
        rec.put("argv", new ArrayList<>(List.of(cmd.isEmpty() ? new String[0] : cmd.split("\\s+"))));
        rec.put("exe", ProcessHandle.current().info().command().orElse(null));
        rec.put("runtime", "jvm");
        rec.put("x", x);
        rec.put("t_ms", ManagementFactory.getRuntimeMXBean().getUptime());
        rec.put("gc_forced", !tick);

        String purge = env("DLA_PURGE");
        boolean wantPurge = last && !tick && purge != null && !purge.equals("none");
        Map<String, Object> preGc = new LinkedHashMap<>();
        Map<String, Object> osBeforeGc = null;
        if (!tick) {
            preGc.putAll(jvmHeap());
            preGc.put("os.rss", osValues().get("os.rss"));
            if (wantPurge && purge.equals("gc")) {
                osBeforeGc = new LinkedHashMap<>(osValues());
                osBeforeGc.put("jvm.heap_used", preGc.get("jvm.heap_used"));
            }
            System.gc();
            System.gc();
        }
        Map<String, Object> values = new LinkedHashMap<>();
        values.putAll(jvmHeap());
        values.putAll(jvmOther(lim));
        values.putAll(osValues());
        rec.put("values", values);
        if (!tick) {
            rec.put("values_pre_gc", preGc);
        }
        if (wantPurge) {
            rec.put("purge", purge(purge, osBeforeGc, values, lim));
        }
        if (tick) {
            return;
        }
        rec.put("probes", probes(lim));
        rec.put("types", types(lim));
        if (phase.equals("exit")) {
            Map<String, Object> cov = coverage(lim);
            if (cov != null) {
                rec.put("coverage", cov);
            }
        }
        lim.add("jvm probe bytes are a shallow estimate assuming compressed oops");
        if (DlaCapture.enabled()) {
            List<Object> caps = DlaCapture.capture(phase, last, lim);
            if (!caps.isEmpty()) {
                rec.put("captures", caps);
            }
        }
    }

    // ------------------------------------------------------------------ values

    private static Map<String, Object> jvmHeap() {
        Map<String, Object> v = new LinkedHashMap<>();
        MemoryMXBean mem = ManagementFactory.getMemoryMXBean();
        v.put("jvm.heap_used", mem.getHeapMemoryUsage().getUsed());
        v.put("jvm.heap_committed", mem.getHeapMemoryUsage().getCommitted());
        return v;
    }

    private static Map<String, Object> jvmOther(List<String> lim) {
        Map<String, Object> v = new LinkedHashMap<>();
        MemoryMXBean mem = ManagementFactory.getMemoryMXBean();
        v.put("jvm.nonheap_used", mem.getNonHeapMemoryUsage().getUsed());
        v.put("jvm.nonheap_committed", mem.getNonHeapMemoryUsage().getCommitted());
        v.put("jvm.threads", ManagementFactory.getThreadMXBean().getThreadCount());
        ClassLoadingMXBean cl = ManagementFactory.getClassLoadingMXBean();
        v.put("jvm.classes_loaded", cl.getLoadedClassCount());
        try {
            String nmt = diag("vmNativeMemory", "summary");
            Matcher m = Pattern.compile("Total:\\s+reserved=(\\d+)(\\w+),\\s+committed=(\\d+)(\\w+)").matcher(nmt);
            if (m.find()) {
                v.put("jvm.nmt_committed", Long.parseLong(m.group(3)) * unit(m.group(4)));
            } else {
                lim.add("NMT off: jvm.nmt_committed unavailable (run with -XX:NativeMemoryTracking=summary)");
            }
        } catch (Throwable t) {
            lim.add("NMT unavailable: " + t);
        }
        return v;
    }

    private static long unit(String u) {
        return switch (u.toUpperCase()) {
            case "KB" -> 1024L;
            case "MB" -> 1024L * 1024;
            case "GB" -> 1024L * 1024 * 1024;
            default -> 1L;
        };
    }

    private static String diag(String op, String... args) throws Exception {
        MBeanServer s = ManagementFactory.getPlatformMBeanServer();
        ObjectName n = new ObjectName("com.sun.management:type=DiagnosticCommand");
        return (String) s.invoke(n, op, new Object[] {args}, new String[] {String[].class.getName()});
    }

    /** Reads os.* values from /proc/self/status (empty on other platforms). */
    public static Map<String, Object> osValues() {
        Map<String, Object> v = new LinkedHashMap<>();
        try {
            Map<String, String> keys = Map.of("VmRSS", "os.rss", "RssAnon", "os.rss_anon", "RssFile", "os.rss_file",
                "RssShmem", "os.rss_shmem", "VmSwap", "os.swap", "VmData", "os.vm_data", "VmHWM", "os.vm_hwm");
            for (String line : Files.readAllLines(Path.of("/proc/self/status"))) {
                int c = line.indexOf(':');
                if (c < 0) {
                    continue;
                }
                String k = line.substring(0, c);
                String rest = line.substring(c + 1).trim();
                if (keys.containsKey(k)) {
                    v.put(keys.get(k), Long.parseLong(rest.split("\\s+")[0]) * 1024);
                } else if (k.equals("Threads")) {
                    v.put("os.threads", Long.parseLong(rest));
                }
            }
        } catch (Throwable ignored) {
            // not Linux or unreadable
        }
        return v;
    }

    private static Map<String, Object> purge(String kind, Map<String, Object> osBeforeGc, Map<String, Object> afterGc,
            List<String> lim) {
        Map<String, Object> p = new LinkedHashMap<>();
        p.put("kind", kind);
        switch (kind) {
            case "gc" -> {
                p.put("before", osBeforeGc == null ? new LinkedHashMap<>() : osBeforeGc);
                Map<String, Object> after = new LinkedHashMap<>(osValues());
                after.put("jvm.heap_used", afterGc.get("jvm.heap_used"));
                p.put("after", after);
            }
            case "trim-native-heap", "trim" -> {
                Map<String, Object> before = new LinkedHashMap<>(osValues());
                p.put("before", before);
                try {
                    p.put("detail", diag("systemTrimNativeHeap").trim());
                } catch (Throwable t) {
                    lim.add("systemTrimNativeHeap failed: " + t);
                }
                p.put("after", new LinkedHashMap<>(osValues()));
            }
            default -> lim.add("purge '" + kind + "' is not supported by the jvm agent");
        }
        return p;
    }

    // ------------------------------------------------------------------ class histogram and coverage

    private static List<Object> types(List<String> lim) {
        List<Object> out = new ArrayList<>();
        try {
            String text = diag("gcClassHistogram");
            Pattern row = Pattern.compile("^\\s*\\d+:\\s+(\\d+)\\s+(\\d+)\\s+(\\S+)");
            List<long[]> rows = new ArrayList<>();
            List<String> names = new ArrayList<>();
            for (String line : text.split("\\R")) {
                Matcher m = row.matcher(line);
                if (m.find()) {
                    rows.add(new long[] {Long.parseLong(m.group(1)), Long.parseLong(m.group(2)), names.size()});
                    names.add(m.group(3));
                }
            }
            rows.sort(Comparator.<long[]>comparingLong(r -> -r[0]).thenComparingLong(r -> -r[1]));
            for (int i = 0; i < Math.min(TYPES_TOP, rows.size()); i++) {
                long[] r = rows.get(i);
                Map<String, Object> t = new LinkedHashMap<>();
                t.put("name", typeName(names.get((int) r[2])));
                t.put("count", r[0]);
                t.put("bytes", r[1]);
                out.add(t);
            }
        } catch (Throwable t) {
            lim.add("class histogram unavailable: " + t);
        }
        return out;
    }

    private static String typeName(String raw) {
        int dims = 0;
        while (dims < raw.length() && raw.charAt(dims) == '[') {
            dims++;
        }
        if (dims == 0) {
            return raw;
        }
        String base = raw.substring(dims);
        String name = switch (base) {
            case "B" -> "byte";
            case "C" -> "char";
            case "D" -> "double";
            case "F" -> "float";
            case "I" -> "int";
            case "J" -> "long";
            case "S" -> "short";
            case "Z" -> "boolean";
            default -> base.startsWith("L") && base.endsWith(";") ? base.substring(1, base.length() - 1) : base;
        };
        return name + "[]".repeat(dims);
    }

    private static Map<String, Object> coverage(List<String> lim) {
        Instrumentation in = inst;
        if (in == null) {
            lim.add("coverage unavailable: no instrumentation handle");
            return null;
        }
        Path project = projectRoot();
        List<String> own = new ArrayList<>();
        List<String> other = new ArrayList<>();
        for (Class<?> c : in.getAllLoadedClasses()) {
            try {
                String n = c.getName();
                if (c.isArray() || c.isHidden() || n.startsWith("dla.harness.") || n.contains("$$Lambda")) {
                    continue;
                }
                ClassLoader l = c.getClassLoader();
                if (l == null || l == ClassLoader.getPlatformClassLoader()) {
                    continue;
                }
                if (n.startsWith("java.") || n.startsWith("jdk.") || n.startsWith("sun.")) {
                    continue;
                }
                if (project != null && underProject(c, project)) {
                    own.add(n);
                } else {
                    other.add(n);
                }
            } catch (Throwable ignored) {
                // skip this class
            }
        }
        own.sort(null);
        other.sort(null);
        List<Object> items = new ArrayList<>(own);
        for (String n : other) {
            if (items.size() >= COVERAGE_CAP) {
                lim.add("coverage capped at " + COVERAGE_CAP + " classes");
                break;
            }
            items.add(n);
        }
        if (items.size() > COVERAGE_CAP) {
            items = new ArrayList<>(items.subList(0, COVERAGE_CAP));
            lim.add("coverage capped at " + COVERAGE_CAP + " classes");
        }
        Map<String, Object> cov = new LinkedHashMap<>();
        cov.put("kind", "classes");
        cov.put("items", items);
        return cov;
    }

    private static Path projectRoot() {
        String p = env("DLA_PROJECT");
        return p == null ? null : Path.of(p).toAbsolutePath().normalize();
    }

    private static boolean underProject(Class<?> c, Path project) throws Exception {
        var cs = c.getProtectionDomain().getCodeSource();
        if (cs == null || cs.getLocation() == null || !"file".equals(cs.getLocation().getProtocol())) {
            return false;
        }
        return Path.of(cs.getLocation().toURI()).toAbsolutePath().normalize().startsWith(project);
    }

    // ------------------------------------------------------------------ probes

    private static List<Object> probes(List<String> lim) {
        List<Object> out = new ArrayList<>();
        String file = env("DLA_HARNESS_PROBES");
        if (file == null) {
            return out;
        }
        Object parsed;
        try {
            parsed = Json.read(Files.readString(Path.of(file), StandardCharsets.UTF_8));
        } catch (Throwable t) {
            lim.add("probes.json unreadable: " + t);
            return out;
        }
        List<?> specs = parsed instanceof Map<?, ?> m && m.get("probes") instanceof List<?> l ? l
            : parsed instanceof List<?> l2 ? l2 : List.of();
        Map<String, Class<?>> loaded = null;
        for (Object o : specs) {
            if (!(o instanceof Map<?, ?> spec)) {
                continue;
            }
            String id = str(spec.get("id"));
            try {
                Object tgt = spec.get("target") instanceof Map<?, ?> t ? t : spec;
                Map<?, ?> target = (Map<?, ?>) tgt;
                String kind = str(target.get("kind")).replaceAll("[_\\-]", "").toLowerCase();
                if (!kind.startsWith("jvm")) {
                    continue; // another runtime's probe
                }
                if (loaded == null) {
                    loaded = loadedClasses();
                }
                String cname = str(target.get("class"));
                Class<?> c = findClass(cname, loaded);
                if (c == null) {
                    out.add(probeRecord(id, cname, cname, null, 0, 0, null, "class not loaded: " + cname));
                } else if (kind.equals("jvmclass")) {
                    for (Field f : c.getDeclaredFields()) {
                        if (probeField(f)) {
                            out.add(probeOne(id, c, f, lim));
                        }
                    }
                } else if (kind.equals("jvmstatic")) {
                    String fname = str(target.get("field"));
                    Field f = null;
                    try {
                        f = c.getDeclaredField(fname);
                    } catch (NoSuchFieldException e) {
                        out.add(probeRecord(id, fname, cname + "." + fname, null, 0, 0, null, "no such field"));
                    }
                    if (f != null) {
                        out.add(probeOne(id, c, f, lim));
                    }
                }
            } catch (Throwable t) {
                out.add(probeRecord(id, null, null, null, 0, 0, null, t.toString()));
            }
        }
        return out;
    }

    private static String str(Object o) {
        return o == null ? "" : o.toString();
    }

    private static Map<String, Class<?>> loadedClasses() {
        Map<String, Class<?>> m = new HashMap<>();
        Instrumentation in = inst;
        if (in != null) {
            for (Class<?> c : in.getAllLoadedClasses()) {
                if (c.getClassLoader() != null) {
                    m.putIfAbsent(c.getName(), c);
                }
            }
        }
        return m;
    }

    private static Class<?> findClass(String name, Map<String, Class<?>> loaded) {
        Class<?> c = loaded.get(name);
        if (c != null) {
            return c;
        }
        ClassLoader[] loaders = {Thread.currentThread().getContextClassLoader(), ClassLoader.getSystemClassLoader()};
        for (ClassLoader l : loaders) {
            try {
                return Class.forName(name, false, l);
            } catch (Throwable ignored) {
                // try the next loader
            }
        }
        return null;
    }

    private static boolean probeField(Field f) {
        if (!Modifier.isStatic(f.getModifiers()) || f.isSynthetic()) {
            return false;
        }
        Class<?> t = f.getType();
        return !(t.isPrimitive() || t == String.class || t == Class.class || t.isEnum()
            || Number.class.isAssignableFrom(t) || t == Boolean.class || t == Character.class);
    }

    private static Map<String, Object> probeOne(String id, Class<?> c, Field f, List<String> lim) {
        String path = c.getName() + "." + f.getName();
        try {
            f.setAccessible(true);
            Object v = f.get(null);
            if (v instanceof ThreadLocal<?> tl) {
                return threadLocalProbe(id, f, path, tl, lim);
            }
            Long size = sizeOf(v);
            long[] w = walk(v);
            if (w[2] != 0) {
                lim.add("probe " + path + " walk truncated at " + MAX_OBJECTS + " objects");
            }
            return probeRecord(id, f.getName(), path, size, w[0], w[1], null, null);
        } catch (Throwable t) {
            return probeRecord(id, f.getName(), path, null, 0, 0, null, t.toString());
        }
    }

    /**
     * A ThreadLocal field is measured through its per-thread values: the sum over all live threads
     * of the value this ThreadLocal holds in the thread's threadLocals and inheritableThreadLocals
     * maps (size: collection, map or array length; reachable and bytes: one bounded walk from all
     * the values). When the thread maps cannot be read the probe abstains with an error and adds a
     * limitation, so an empty measurement never refutes.
     */
    private static Map<String, Object> threadLocalProbe(String id, Field f, String path, ThreadLocal<?> tl,
            List<String> lim) {
        try {
            List<Object> values = new ArrayList<>();
            for (Thread t : Thread.getAllStackTraces().keySet()) {
                for (String mapField : new String[] {"threadLocals", "inheritableThreadLocals"}) {
                    Object v = threadLocalValue(t, mapField, tl);
                    if (v != null) {
                        values.add(v);
                    }
                }
            }
            long sum = 0;
            boolean sized = false;
            Walker w = new Walker();
            for (Object v : values) {
                Long s = sizeOf(v);
                if (s != null) {
                    sum += s;
                    sized = true;
                }
                w.add(v);
            }
            w.run();
            if (w.truncated) {
                lim.add("probe " + path + " walk truncated at " + MAX_OBJECTS + " objects");
            }
            return probeRecord(id, f.getName(), path, sized ? Long.valueOf(sum) : null, w.count, w.bytes, null, null);
        } catch (Throwable t) {
            String why = "thread-local values unreadable: " + t;
            lim.add("probe " + path + ": " + why + " (needs --add-opens java.base/java.lang)");
            return probeRecord(id, f.getName(), path, null, 0, 0, null, why);
        }
    }

    /** The value of tl in thread t's map (null when absent); throws when the JDK internals are closed. */
    private static Object threadLocalValue(Thread t, String mapField, ThreadLocal<?> tl) throws Exception {
        Field mf = Thread.class.getDeclaredField(mapField);
        mf.setAccessible(true);
        Object map = mf.get(t);
        if (map == null) {
            return null;
        }
        Field tf = map.getClass().getDeclaredField("table");
        tf.setAccessible(true);
        Object[] table = (Object[]) tf.get(map);
        if (table == null) {
            return null;
        }
        Field vf = null;
        for (Object e : table) {
            if (e == null) {
                continue;
            }
            if (vf == null) {
                vf = e.getClass().getDeclaredField("value");
                vf.setAccessible(true);
            }
            if (((java.lang.ref.Reference<?>) e).get() == tl) {
                return vf.get(e);
            }
        }
        return null;
    }

    private static Map<String, Object> probeRecord(String id, String name, String path, Long size, long reachable,
            long bytes, Object unused, String error) {
        Map<String, Object> r = new LinkedHashMap<>();
        r.put("id", id);
        r.put("name", name);
        r.put("path", path);
        r.put("size", size);
        r.put("reachable", error == null ? reachable : null);
        r.put("bytes", error == null ? bytes : null);
        r.put("error", error);
        return r;
    }

    private static Long sizeOf(Object v) {
        if (v == null) {
            return 0L;
        }
        if (v instanceof Collection<?> c) {
            return (long) c.size();
        }
        if (v instanceof Map<?, ?> m) {
            return (long) m.size();
        }
        if (v.getClass().isArray()) {
            return (long) java.lang.reflect.Array.getLength(v);
        }
        return null;
    }

    // ------------------------------------------------------------------ bounded reachable walk

    private static final Map<Class<?>, Field[]> FIELDS = new HashMap<>();
    private static final Map<Class<?>, Long> SHALLOW = new HashMap<>();

    /** Returns {objects, shallow bytes, truncated(0|1)} reachable from root. Iterative, bounded. */
    static long[] walk(Object root) {
        Walker w = new Walker();
        w.add(root);
        w.run();
        return new long[] {w.count, w.bytes, w.truncated ? 1 : 0};
    }

    private static final class Walker {
        final IdentityHashMap<Object, Object> seen = new IdentityHashMap<>();
        final ArrayDeque<Object> stack = new ArrayDeque<>();
        long count;
        long bytes;
        boolean truncated;

        void add(Object o) {
            if (o == null || skip(o) || seen.put(o, PRESENT) != null) {
                return;
            }
            if (count >= MAX_OBJECTS) {
                truncated = true;
                return;
            }
            count++;
            bytes += shallow(o);
            if (descend(o)) {
                stack.push(o);
            }
        }

        void run() {
            while (!stack.isEmpty() && !truncated) {
                Object o = stack.pop();
                try {
                    expand(o);
                } catch (Throwable ignored) {
                    // concurrent modification or inaccessible state: keep what was counted
                }
            }
        }

        void expand(Object o) throws Exception {
            if (o instanceof Object[] arr) {
                for (Object e : arr) {
                    add(e);
                }
                return;
            }
            if (o instanceof Map<?, ?> m) {
                for (Map.Entry<?, ?> e : m.entrySet()) {
                    add(e.getKey());
                    add(e.getValue());
                }
            } else if (o instanceof Collection<?> c) {
                for (Object e : c) {
                    add(e);
                }
            } else if (o instanceof Optional<?> opt) {
                add(opt.orElse(null));
            } else if (o instanceof AtomicReference<?> ref) {
                add(ref.get());
            } else if (o instanceof Map.Entry<?, ?> e) {
                add(e.getKey());
                add(e.getValue());
            }
            for (Field f : fieldsOf(o.getClass())) {
                add(f.get(o));
            }
        }
    }

    private static boolean skip(Object o) {
        return o instanceof Class<?> || o instanceof ClassLoader || o instanceof Thread || o instanceof ThreadGroup
            || o instanceof AccessibleObject || o instanceof Module || o instanceof java.security.ProtectionDomain
            || o.getClass().getName().startsWith("java.lang.invoke.");
    }

    private static boolean descend(Object o) {
        Class<?> c = o.getClass();
        if (c.isArray()) {
            return !c.getComponentType().isPrimitive();
        }
        return !(o instanceof Enum<?> || o instanceof Reference<?> || o instanceof CharSequence
            || o instanceof Number || o instanceof Boolean || o instanceof Character);
    }

    private static boolean isJdk(Class<?> c) {
        Module m = c.getModule();
        if (!m.isNamed()) {
            return false;
        }
        String n = m.getName();
        return n.startsWith("java.") || n.startsWith("jdk.");
    }

    /** Accessible reference instance fields of user classes in the hierarchy (JDK classes are walked by API). */
    private static Field[] fieldsOf(Class<?> c) {
        Field[] cached = FIELDS.get(c);
        if (cached != null) {
            return cached;
        }
        List<Field> out = new ArrayList<>();
        for (Class<?> k = c; k != null && k != Object.class; k = k.getSuperclass()) {
            if (isJdk(k)) {
                continue;
            }
            for (Field f : k.getDeclaredFields()) {
                if (Modifier.isStatic(f.getModifiers()) || f.getType().isPrimitive()) {
                    continue;
                }
                try {
                    f.setAccessible(true);
                    out.add(f);
                } catch (Throwable ignored) {
                    // not accessible: leaf
                }
            }
        }
        Field[] arr = out.toArray(new Field[0]);
        FIELDS.put(c, arr);
        return arr;
    }

    private static long align(long n) {
        return (n + 7) & ~7L;
    }

    private static long instanceSize(Class<?> c) {
        Long cached = SHALLOW.get(c);
        if (cached != null) {
            return cached;
        }
        long size = 12;
        for (Class<?> k = c; k != null; k = k.getSuperclass()) {
            for (Field f : k.getDeclaredFields()) {
                if (Modifier.isStatic(f.getModifiers())) {
                    continue;
                }
                Class<?> t = f.getType();
                size += t == long.class || t == double.class ? 8
                    : t == int.class || t == float.class ? 4
                    : t == short.class || t == char.class ? 2
                    : t == byte.class || t == boolean.class ? 1 : 4;
            }
        }
        size = align(size);
        SHALLOW.put(c, size);
        return size;
    }

    /** Cheap shallow size estimate (compressed oops): instance layout plus array, string and container payload. */
    private static long shallow(Object o) {
        Class<?> c = o.getClass();
        if (c.isArray()) {
            Class<?> t = c.getComponentType();
            long len = java.lang.reflect.Array.getLength(o);
            long el = t == long.class || t == double.class ? 8
                : t == int.class || t == float.class ? 4
                : t == short.class || t == char.class ? 2
                : t == byte.class || t == boolean.class ? 1 : 4;
            return align(16 + len * el);
        }
        long size = instanceSize(c);
        if (o instanceof String s) {
            size += align(16 + s.length());
        } else if (o instanceof Map<?, ?> m) {
            size += 40L * m.size();
        } else if (o instanceof Collection<?> col) {
            size += (o instanceof java.util.Set<?> ? 40L : o instanceof List<?> ? 4L : 16L) * col.size();
        }
        return size;
    }

    // ------------------------------------------------------------------ tiny JSON

    /** Minimal JSON writer and reader: Map, List, String, Number, Boolean, null. */
    public static final class Json {
        private Json() {}

        /** Serializes a tree of Map, Iterable, array, String, Number, Boolean and null. */
        public static String write(Object o) {
            StringBuilder sb = new StringBuilder();
            write(sb, o);
            return sb.toString();
        }

        private static void write(StringBuilder sb, Object o) {
            if (o == null) {
                sb.append("null");
            } else if (o instanceof String s) {
                quote(sb, s);
            } else if (o instanceof Double || o instanceof Float) {
                double d = ((Number) o).doubleValue();
                sb.append(Double.isNaN(d) || Double.isInfinite(d) ? "null" : o.toString());
            } else if (o instanceof Number || o instanceof Boolean) {
                sb.append(o);
            } else if (o instanceof Map<?, ?> m) {
                sb.append('{');
                boolean first = true;
                for (Map.Entry<?, ?> e : m.entrySet()) {
                    if (!first) {
                        sb.append(',');
                    }
                    first = false;
                    quote(sb, String.valueOf(e.getKey()));
                    sb.append(':');
                    write(sb, e.getValue());
                }
                sb.append('}');
            } else if (o instanceof Iterable<?> it) {
                sb.append('[');
                boolean first = true;
                for (Object e : it) {
                    if (!first) {
                        sb.append(',');
                    }
                    first = false;
                    write(sb, e);
                }
                sb.append(']');
            } else {
                quote(sb, o.toString());
            }
        }

        private static void quote(StringBuilder sb, String s) {
            sb.append('"');
            for (int i = 0; i < s.length(); i++) {
                char c = s.charAt(i);
                switch (c) {
                    case '"' -> sb.append("\\\"");
                    case '\\' -> sb.append("\\\\");
                    case '\n' -> sb.append("\\n");
                    case '\r' -> sb.append("\\r");
                    case '\t' -> sb.append("\\t");
                    default -> {
                        if (c < 0x20) {
                            sb.append(String.format("\\u%04x", (int) c));
                        } else {
                            sb.append(c);
                        }
                    }
                }
            }
            sb.append('"');
        }

        /** Parses JSON text into Map, List, String, Long or Double, Boolean, null. */
        public static Object read(String text) throws IOException {
            int[] pos = {0};
            Object v = value(text, pos);
            skipWs(text, pos);
            if (pos[0] != text.length()) {
                throw new IOException("trailing JSON at " + pos[0]);
            }
            return v;
        }

        private static void skipWs(String s, int[] p) {
            while (p[0] < s.length() && Character.isWhitespace(s.charAt(p[0]))) {
                p[0]++;
            }
        }

        private static Object value(String s, int[] p) throws IOException {
            skipWs(s, p);
            if (p[0] >= s.length()) {
                throw new IOException("unexpected end of JSON");
            }
            char c = s.charAt(p[0]);
            if (c == '{') {
                p[0]++;
                Map<String, Object> m = new LinkedHashMap<>();
                skipWs(s, p);
                if (s.charAt(p[0]) == '}') {
                    p[0]++;
                    return m;
                }
                while (true) {
                    skipWs(s, p);
                    String k = string(s, p);
                    skipWs(s, p);
                    expect(s, p, ':');
                    m.put(k, value(s, p));
                    skipWs(s, p);
                    if (s.charAt(p[0]) == ',') {
                        p[0]++;
                    } else {
                        expect(s, p, '}');
                        return m;
                    }
                }
            }
            if (c == '[') {
                p[0]++;
                List<Object> l = new ArrayList<>();
                skipWs(s, p);
                if (s.charAt(p[0]) == ']') {
                    p[0]++;
                    return l;
                }
                while (true) {
                    l.add(value(s, p));
                    skipWs(s, p);
                    if (s.charAt(p[0]) == ',') {
                        p[0]++;
                    } else {
                        expect(s, p, ']');
                        return l;
                    }
                }
            }
            if (c == '"') {
                return string(s, p);
            }
            if (s.startsWith("true", p[0])) {
                p[0] += 4;
                return Boolean.TRUE;
            }
            if (s.startsWith("false", p[0])) {
                p[0] += 5;
                return Boolean.FALSE;
            }
            if (s.startsWith("null", p[0])) {
                p[0] += 4;
                return null;
            }
            int start = p[0];
            while (p[0] < s.length() && "+-0123456789.eE".indexOf(s.charAt(p[0])) >= 0) {
                p[0]++;
            }
            String num = s.substring(start, p[0]);
            if (num.isEmpty()) {
                throw new IOException("bad JSON at " + start);
            }
            return num.matches("-?\\d+") ? (Object) Long.valueOf(num) : (Object) Double.valueOf(num);
        }

        private static void expect(String s, int[] p, char c) throws IOException {
            if (p[0] >= s.length() || s.charAt(p[0]) != c) {
                throw new IOException("expected '" + c + "' at " + p[0]);
            }
            p[0]++;
        }

        private static String string(String s, int[] p) throws IOException {
            expect(s, p, '"');
            StringBuilder sb = new StringBuilder();
            while (p[0] < s.length()) {
                char c = s.charAt(p[0]++);
                if (c == '"') {
                    return sb.toString();
                }
                if (c == '\\') {
                    char e = s.charAt(p[0]++);
                    switch (e) {
                        case 'n' -> sb.append('\n');
                        case 'r' -> sb.append('\r');
                        case 't' -> sb.append('\t');
                        case 'b' -> sb.append('\b');
                        case 'f' -> sb.append('\f');
                        case 'u' -> {
                            sb.append((char) Integer.parseInt(s.substring(p[0], p[0] + 4), 16));
                            p[0] += 4;
                        }
                        default -> sb.append(e);
                    }
                } else {
                    sb.append(c);
                }
            }
            throw new IOException("unterminated string");
        }
    }
}
