package exports;

/** Starts and completes export jobs; completion hands the page buffers back for the next job. */
final class Exporter {
    private final BufferPool pool;
    private long completed;
    private long pagesWritten;

    Exporter(BufferPool pool) {
        this.pool = pool;
    }

    ExportJob start(Document document) {
        return new ExportJob(document, pool);
    }

    long complete(ExportJob job) {
        long checksum = job.checksum();
        pagesWritten += job.document().pageCount();
        completed++;
        job.giveBack(pool);
        return checksum;
    }

    long completed() {
        return completed;
    }

    long pagesWritten() {
        return pagesWritten;
    }

    BufferPool pool() {
        return pool;
    }
}
