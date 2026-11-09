package billing;

import java.util.ArrayList;
import java.util.List;

/** Per-thread buffer that collects the text segments of the invoice being rendered. */
final class RenderContext {
    private static final ThreadLocal<List<String>> SEGMENTS = ThreadLocal.withInitial(ArrayList::new);

    private RenderContext() {}

    static void append(String segment) {
        SEGMENTS.get().add(segment);
    }

    /** Position to pass to {@link #textFrom} for the segments appended after this call. */
    static int mark() {
        return SEGMENTS.get().size();
    }

    static String textFrom(int mark) {
        List<String> all = SEGMENTS.get();
        return String.join("\n", all.subList(mark, all.size()));
    }

    static void truncate(int mark) {
        List<String> all = SEGMENTS.get();
        all.subList(mark, all.size()).clear();
    }
}
