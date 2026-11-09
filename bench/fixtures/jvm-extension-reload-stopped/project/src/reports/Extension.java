package reports;

/** A separately built module that adds reporting to the host. */
public interface Extension {
    String name();

    void start();

    void stop();

    /** Total in cents that this instance has seen since it started. */
    long total();
}
