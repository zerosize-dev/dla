package billing;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.Callable;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.atomic.AtomicInteger;

/** Renders invoices on a fixed pool of worker threads that lives as long as the service. */
final class InvoiceService {
    private static final int WORKERS = 4;

    private final InvoiceRenderer renderer = new InvoiceRenderer();
    private final ExecutorService pool = Executors.newFixedThreadPool(WORKERS, new WorkerFactory());

    String renderNow(Invoice invoice) {
        return await(pool.submit(() -> renderer.render(invoice)));
    }

    /** Renders a batch and returns the total number of characters produced. */
    long renderBatch(List<Invoice> batch) {
        List<Callable<String>> jobs = new ArrayList<>();
        for (Invoice invoice : batch) {
            jobs.add(() -> renderer.render(invoice));
        }
        long chars = 0;
        try {
            for (Future<String> done : pool.invokeAll(jobs)) {
                chars += await(done).length();
            }
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(e);
        }
        return chars;
    }

    private static String await(Future<String> text) {
        try {
            return text.get();
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(e);
        } catch (ExecutionException e) {
            throw new IllegalStateException(e.getCause());
        }
    }

    /** Daemon workers, so the pool never keeps the process alive; it is never shut down. */
    private static final class WorkerFactory implements java.util.concurrent.ThreadFactory {
        private final AtomicInteger next = new AtomicInteger();

        @Override
        public Thread newThread(Runnable task) {
            Thread worker = new Thread(task, "render-" + next.incrementAndGet());
            worker.setDaemon(true);
            return worker;
        }
    }
}
