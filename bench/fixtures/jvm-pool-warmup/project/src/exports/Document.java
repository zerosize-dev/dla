package exports;

/** A document waiting to be exported: its pages are rendered into caller-supplied buffers. */
final class Document {
    private final int id;
    private final int pageCount;

    Document(int id, int pageCount) {
        if (pageCount < 1) {
            throw new IllegalArgumentException("a document has at least one page");
        }
        this.id = id;
        this.pageCount = pageCount;
    }

    int id() {
        return id;
    }

    int pageCount() {
        return pageCount;
    }

    /** Fills the buffer with the text of one page; returns the number of bytes used. */
    int render(int page, byte[] into) {
        String header = "doc " + id + " page " + (page + 1) + "/" + pageCount + "\n";
        int used = 0;
        for (int i = 0; i < header.length(); i++) {
            into[used++] = (byte) header.charAt(i);
        }
        int line = 0;
        while (used + 40 < into.length && line < 60) {
            int digit = (id * 31 + page * 7 + line) % 10;
            for (int c = 0; c < 39; c++) {
                into[used++] = (byte) ('0' + (digit + c) % 10);
            }
            into[used++] = '\n';
            line++;
        }
        return used;
    }
}
