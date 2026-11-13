package exports;

/** Plain assertions; run with -ea. */
public final class ExporterTest {
    public static void main(String[] args) {
        pagesCarryTheirHeader();
        completingAJobHandsBuffersBack();
        buffersAreReusedByTheNextJob();
        buffersBeyondThePoolAreNotTakenBack();
        checksumDependsOnContent();
        emptyDocumentIsRejected();
        System.out.println("all tests passed");
    }

    private static void pagesCarryTheirHeader() {
        byte[] buffer = new byte[BufferPool.BUFFER_SIZE];
        int used = new Document(12, 3).render(1, buffer);
        String text = new String(buffer, 0, used, java.nio.charset.StandardCharsets.US_ASCII);
        assert text.startsWith("doc 12 page 2/3\n") : text.substring(0, 20);
        assert used > 1000 && used <= buffer.length;
    }

    private static void completingAJobHandsBuffersBack() {
        BufferPool pool = new BufferPool();
        Exporter exporter = new Exporter(pool);
        ExportJob job = exporter.start(new Document(1, 4));
        assert pool.idle() == 0 && pool.owned() == 4;
        exporter.complete(job);
        assert pool.idle() == 4;
        assert exporter.completed() == 1 && exporter.pagesWritten() == 4;
    }

    private static void buffersAreReusedByTheNextJob() {
        BufferPool pool = new BufferPool();
        Exporter exporter = new Exporter(pool);
        exporter.complete(exporter.start(new Document(1, 5)));
        exporter.complete(exporter.start(new Document(2, 5)));
        assert pool.owned() == 5 : pool.owned();
    }

    private static void buffersBeyondThePoolAreNotTakenBack() {
        BufferPool pool = new BufferPool();
        Exporter exporter = new Exporter(pool);
        ExportJob big = exporter.start(new Document(1, BufferPool.SIZE + 5));
        assert pool.owned() == BufferPool.SIZE && pool.unpooled() == 5;
        exporter.complete(big);
        assert pool.idle() == BufferPool.SIZE;
    }

    private static void checksumDependsOnContent() {
        Exporter exporter = new Exporter(new BufferPool());
        long a = exporter.complete(exporter.start(new Document(1, 2)));
        long b = exporter.complete(exporter.start(new Document(2, 2)));
        long again = exporter.complete(exporter.start(new Document(1, 2)));
        assert a != b;
        assert a == again;
    }

    private static void emptyDocumentIsRejected() {
        try {
            new Document(3, 0);
            assert false : "expected a rejection";
        } catch (IllegalArgumentException expected) {
            assert expected.getMessage().contains("page");
        }
    }
}
