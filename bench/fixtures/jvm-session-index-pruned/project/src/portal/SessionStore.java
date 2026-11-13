package portal;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Holds the open sessions, by id for requests and by user for "sign out everywhere". */
final class SessionStore {
    private static final Map<String, Session> BY_ID = new HashMap<>();
    private static final Map<String, List<Session>> BY_USER = new HashMap<>();
    private static long sequence;

    private SessionStore() {}

    static synchronized Session open(String userId, String device, long now) {
        Session session = new Session("s-" + Long.toString(++sequence, 36), userId, device, now);
        BY_ID.put(session.id(), session);
        BY_USER.computeIfAbsent(userId, key -> new ArrayList<>()).add(session);
        return session;
    }

    static synchronized Session find(String sessionId) {
        return BY_ID.get(sessionId);
    }

    /** Ends one session; returns false when it was not open. */
    static synchronized boolean close(String sessionId) {
        Session session = BY_ID.remove(sessionId);
        if (session == null) {
            return false;
        }
        List<Session> sessions = BY_USER.get(session.userId());
        if (sessions != null) {
            sessions.remove(session);
            if (sessions.isEmpty()) {
                BY_USER.remove(session.userId());
            }
        }
        return true;
    }

    /** Ends every open session of a user; returns how many were open. */
    static synchronized int closeAllFor(String userId) {
        List<Session> sessions = BY_USER.remove(userId);
        int closed = 0;
        if (sessions != null) {
            for (Session session : sessions) {
                if (BY_ID.remove(session.id()) != null) {
                    closed++;
                }
            }
        }
        return closed;
    }

    /** Used by tests that need a clean slate. */
    static synchronized void clear() {
        BY_ID.clear();
        BY_USER.clear();
    }

    static synchronized int openCount() {
        return BY_ID.size();
    }
}
