package portal;

/** Sign-in, sign-out and the operator's "sign this user out everywhere" action. */
final class SessionService {
    private final Directory directory;
    private int signedIn;
    private int signedOut;
    private int removedByOperator;

    SessionService(Directory directory) {
        this.directory = directory;
    }

    Session login(String userId, String device, long now) {
        if (!directory.exists(userId)) {
            throw new IllegalArgumentException("unknown user " + userId);
        }
        Session session = SessionStore.open(userId, device, now);
        session.setPreference("theme", (userId.length() % 2 == 0) ? "dark" : "light");
        session.setPreference("locale", "en-" + (device.length() % 3 == 0 ? "GB" : "US"));
        signedIn++;
        return session;
    }

    boolean logout(String sessionId) {
        boolean closed = SessionStore.close(sessionId);
        if (closed) {
            signedOut++;
        }
        return closed;
    }

    /** Operator action: ends all open sessions of one user. */
    int signOutEverywhere(String userId) {
        int closed = SessionStore.closeAllFor(userId);
        removedByOperator += closed;
        return closed;
    }

    Session request(String sessionId, long now, String page) {
        Session session = SessionStore.find(sessionId);
        if (session != null) {
            session.touch(now, page);
        }
        return session;
    }

    int signedIn() {
        return signedIn;
    }

    int signedOut() {
        return signedOut;
    }

    int removedByOperator() {
        return removedByOperator;
    }

    int openSessions() {
        return SessionStore.openCount();
    }
}
