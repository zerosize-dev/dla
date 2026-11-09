package reports;

import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;

/** Loads the reporting extension from a directory and swaps in a fresh copy on demand. */
public final class ExtensionManager {
    private static final String ENTRY = "ledger.LedgerExtension";

    private final URL location;
    private URLClassLoader loader;
    private Extension current;

    public ExtensionManager(Path dir) throws Exception {
        this.location = dir.toUri().toURL();
    }

    public void load() throws Exception {
        loader = new URLClassLoader(new URL[] {location}, ExtensionManager.class.getClassLoader());
        Class<?> type = Class.forName(ENTRY, true, loader);
        current = (Extension) type.getDeclaredConstructor().newInstance();
        current.start();
    }

    /** Picks up a rebuilt extension without restarting the process. */
    public void reload() throws Exception {
        if (current != null) {
            current.stop();
        }
        loader = null;
        current = null;
        load();
    }

    public Extension current() {
        return current;
    }

    public void shutdown() {
        if (current != null) {
            current.stop();
        }
        current = null;
        loader = null;
    }
}
