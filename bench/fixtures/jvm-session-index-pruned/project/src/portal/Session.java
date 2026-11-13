package portal;

import java.util.LinkedHashMap;
import java.util.Map;

/** One signed-in browser or device of a user. */
final class Session {
    private final String id;
    private final String userId;
    private final String device;
    private final long openedAt;
    private final Map<String, String> preferences = new LinkedHashMap<>();
    private final byte[] outbox = new byte[1536];
    private long lastSeen;
    private int requests;

    Session(String id, String userId, String device, long openedAt) {
        this.id = id;
        this.userId = userId;
        this.device = device;
        this.openedAt = openedAt;
        this.lastSeen = openedAt;
    }

    String id() {
        return id;
    }

    String userId() {
        return userId;
    }

    String device() {
        return device;
    }

    long openedAt() {
        return openedAt;
    }

    int requests() {
        return requests;
    }

    /** Records a request and queues a short notice in the outbox. */
    void touch(long now, String page) {
        lastSeen = now;
        outbox[requests % outbox.length] = (byte) page.length();
        requests++;
    }

    void setPreference(String key, String value) {
        preferences.put(key, value);
    }

    String preference(String key) {
        return preferences.get(key);
    }

    long lastSeen() {
        return lastSeen;
    }
}
