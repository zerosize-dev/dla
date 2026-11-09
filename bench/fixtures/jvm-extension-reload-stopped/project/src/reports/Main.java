package reports;

import java.lang.management.ManagementFactory;
import java.nio.file.Path;
import java.util.Random;

/** Load driver: reloads the extension the given number of times, publishing sales in between. */
public final class Main {
    public static void main(String[] args) throws Exception {
        Path dir = Path.of(args[0]);
        int reloads = args.length > 1 ? Integer.parseInt(args[1]) : 2000;
        ExtensionManager manager = new ExtensionManager(dir);
        manager.load();
        Random random = new Random(11);
        long orderId = 1;
        int every = Math.max(1, reloads / 4);
        for (int i = 1; i <= reloads; i++) {
            manager.reload();
            if (i % 8 == 0) {
                EventBus.publish(new Event(orderId++, "r" + random.nextInt(8), 100 + random.nextInt(5000)));
            }
            if (i % every == 0 || i == reloads) {
                long heap = usedKb();
                System.out.printf("reloads=%d listeners=%d classesLoaded=%d total=%d heapKB=%d%n",
                        i, EventBus.listenerCount(), loadedClasses(), manager.current().total(), heap);
            }
        }
        manager.shutdown();
    }

    private static int loadedClasses() {
        return ManagementFactory.getClassLoadingMXBean().getLoadedClassCount();
    }

    private static long usedKb() {
        Runtime rt = Runtime.getRuntime();
        System.gc();
        return (rt.totalMemory() - rt.freeMemory()) / 1024;
    }
}
