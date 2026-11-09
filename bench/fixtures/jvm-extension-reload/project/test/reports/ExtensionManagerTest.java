package reports;

import java.nio.file.Path;

public final class ExtensionManagerTest {
    public static void main(String[] args) throws Exception {
        ExtensionManager manager = new ExtensionManager(Path.of(args[0]));
        manager.load();
        EventBus.publish(new Event(1, "r1", 500));
        EventBus.publish(new Event(2, "r2", 250));
        check(manager.current().total() == 750, "total after two events");
        manager.reload();
        check(manager.current().total() == 0, "reload starts a fresh instance");
        EventBus.publish(new Event(3, "r1", 100));
        check(manager.current().total() == 100, "fresh instance counts");
        int before = EventBus.listenerCount();
        manager.shutdown();
        check(EventBus.listenerCount() == before - 1, "shutdown stops the running instance");
        System.out.println("all tests passed");
    }

    private static void check(boolean ok, String what) {
        if (!ok) {
            throw new AssertionError(what);
        }
    }
}
