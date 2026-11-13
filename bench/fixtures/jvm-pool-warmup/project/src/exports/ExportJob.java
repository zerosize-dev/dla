package exports;

import java.util.ArrayList;
import java.util.List;

/** One document being exported: holds a buffer per page until the job is completed. */
final class ExportJob {
    private final Document document;
    private final List<BufferPool.Lease> pages = new ArrayList<>();
    private final int[] lengths;

    ExportJob(Document document, BufferPool pool) {
        this.document = document;
        this.lengths = new int[document.pageCount()];
        for (int page = 0; page < document.pageCount(); page++) {
            BufferPool.Lease lease = pool.acquire();
            lengths[page] = document.render(page, lease.bytes());
            pages.add(lease);
        }
    }

    Document document() {
        return document;
    }

    /** A simple checksum over everything rendered. */
    long checksum() {
        long sum = 17;
        for (int page = 0; page < pages.size(); page++) {
            byte[] buffer = pages.get(page).bytes();
            for (int i = 0; i < lengths[page]; i++) {
                sum = sum * 31 + buffer[i];
            }
        }
        return sum;
    }

    void giveBack(BufferPool pool) {
        for (BufferPool.Lease lease : pages) {
            pool.release(lease);
        }
        pages.clear();
    }
}
