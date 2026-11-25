package dla.harness;

import java.lang.instrument.Instrumentation;

/**
 * Premain agent injected through JAVA_TOOL_OPTIONS. It registers a shutdown hook that writes one
 * "exit" record to DLA_HARNESS_OUT (see DlaDump) and, when DLA_TICK_MS is set, a daemon thread
 * that writes light "tick" records. It never throws into the target.
 */
public final class DlaAgent {
    private DlaAgent() {}

    /** Agent entry point. */
    public static void premain(String agentArgs, Instrumentation inst) {
        try {
            if (DlaDump.isToolJvm() || DlaDump.env("DLA_HARNESS_OUT") == null) {
                return;
            }
            DlaDump.setInstrumentation(inst);
            openThreadLocals(inst);
            if (DlaCapture.enabled()) {
                java.util.List<String> lim = new java.util.ArrayList<>();
                DlaCapture.start(lim);
                DlaDump.addLimitations(lim);
            }
            Runtime.getRuntime().addShutdownHook(new Thread(() -> {
                try {
                    // A driver that wrote checkpoints owns its own final record.
                    if (!DlaDump.checkpointWritten()) {
                        DlaDump.dump("exit", DlaDump.envLong("DLA_ITER"), true);
                    }
                } catch (Throwable ignored) {
                    // never disturb the target's shutdown
                }
            }, "dla-agent-exit"));
            Long tick = DlaDump.envLong("DLA_TICK_MS");
            if (tick != null && tick > 0) {
                Thread ticker = new Thread(() -> {
                    while (true) {
                        try {
                            Thread.sleep(tick);
                            DlaDump.dump("tick", null, false);
                        } catch (InterruptedException e) {
                            return;
                        } catch (Throwable ignored) {
                            // keep ticking
                        }
                    }
                }, "dla-agent-tick");
                ticker.setDaemon(true);
                ticker.start();
            }
        } catch (Throwable ignored) {
            // the agent must never crash the target
        }
    }

    /**
     * Opens java.lang to the agent's module so the probes can read the per-thread ThreadLocal maps
     * (the same effect as --add-opens java.base/java.lang=ALL-UNNAMED). Failure only means that
     * ThreadLocal probes abstain.
     */
    private static void openThreadLocals(Instrumentation inst) {
        try {
            Module base = Object.class.getModule();
            Module self = DlaAgent.class.getModule();
            if (!base.isOpen("java.lang", self)) {
                inst.redefineModule(base, java.util.Set.of(), java.util.Map.of(),
                    java.util.Map.of("java.lang", java.util.Set.of(self)), java.util.Set.of(), java.util.Map.of());
            }
        } catch (Throwable ignored) {
            // the ThreadLocal probe abstains when the maps cannot be read
        }
    }
}
