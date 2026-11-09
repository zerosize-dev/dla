package reports;

import java.util.ArrayList;
import java.util.List;

/** Process-wide fan-out of sale events to the registered listeners. */
public final class EventBus {
    private static final List<Listener> LISTENERS = new ArrayList<>();

    private EventBus() {}

    public static synchronized void subscribe(Listener listener) {
        LISTENERS.add(listener);
    }

    public static synchronized void unsubscribe(Listener listener) {
        LISTENERS.remove(listener);
    }

    public static synchronized void publish(Event event) {
        for (Listener l : LISTENERS) {
            l.onEvent(event);
        }
    }

    public static synchronized int listenerCount() {
        return LISTENERS.size();
    }
}
