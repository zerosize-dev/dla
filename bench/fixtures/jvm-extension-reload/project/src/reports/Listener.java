package reports;

/** Receives every event published on the bus. */
public interface Listener {
    void onEvent(Event event);
}
