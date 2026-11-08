package ledger;

import java.lang.ref.WeakReference;
import java.util.ArrayList;
import java.util.List;

/** Holds a known number of entries in a static list; used as a heap dump fixture. */
public final class Journal {
    static final class Entry {
        final int id;
        final String label;

        Entry(int id, String label) {
            this.id = id;
            this.label = label;
        }
    }

    static final List<Entry> ENTRIES = new ArrayList<>();
    static final WeakReference<byte[]> SCRATCH = new WeakReference<>(new byte[65536]);
    static byte[] pin;

    public static void main(String[] args) throws Exception {
        int n = Integer.parseInt(args[0]);
        for (int i = 0; i < n; i++) {
            ENTRIES.add(new Entry(i, "item-" + i));
        }
        pin = SCRATCH.get();
        pin = null;
        System.gc();
        System.out.println("ready " + ENTRIES.size());
        System.out.flush();
        System.in.read();
    }
}
