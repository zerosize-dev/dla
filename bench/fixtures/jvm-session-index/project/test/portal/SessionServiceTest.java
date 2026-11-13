package portal;

/** Plain assertions; run with -ea. */
public final class SessionServiceTest {
    public static void main(String[] args) {
        loginThenRequest();
        logoutEndsOnlyThatSession();
        signOutEverywhereEndsOpenSessions();
        signOutEverywhereSkipsSessionsAlreadyClosed();
        unknownUsersCannotLogin();
        System.out.println("all tests passed");
    }

    private static SessionService service() {
        SessionStore.clear();
        return new SessionService(new Directory(10));
    }

    private static void loginThenRequest() {
        SessionService service = service();
        Session session = service.login("user-3", "phone", 1000L);
        assert service.request(session.id(), 1500L, "/home") == session;
        assert session.requests() == 1 && session.lastSeen() == 1500L;
        assert "dark".equals(session.preference("theme")) == ("user-3".length() % 2 == 0);
    }

    private static void logoutEndsOnlyThatSession() {
        SessionService service = service();
        Session a = service.login("user-1", "phone", 1L);
        Session b = service.login("user-1", "laptop", 2L);
        assert service.logout(a.id());
        assert !service.logout(a.id());
        assert service.request(a.id(), 3L, "/home") == null;
        assert service.request(b.id(), 3L, "/home") == b;
        assert service.openSessions() == 1;
    }

    private static void signOutEverywhereEndsOpenSessions() {
        SessionService service = service();
        Session a = service.login("user-2", "phone", 1L);
        Session b = service.login("user-2", "laptop", 2L);
        Session other = service.login("user-4", "laptop", 3L);
        assert service.signOutEverywhere("user-2") == 2;
        assert service.request(a.id(), 4L, "/home") == null;
        assert service.request(b.id(), 4L, "/home") == null;
        assert service.request(other.id(), 4L, "/home") == other;
        assert service.removedByOperator() == 2;
    }

    private static void signOutEverywhereSkipsSessionsAlreadyClosed() {
        SessionService service = service();
        Session a = service.login("user-5", "phone", 1L);
        service.login("user-5", "laptop", 2L);
        service.logout(a.id());
        assert service.signOutEverywhere("user-5") == 1;
        assert service.signOutEverywhere("user-5") == 0;
        assert service.openSessions() == 0;
    }

    private static void unknownUsersCannotLogin() {
        SessionService service = service();
        try {
            service.login("user-99", "phone", 1L);
            assert false : "expected a rejection";
        } catch (IllegalArgumentException expected) {
            assert service.openSessions() == 0;
        }
    }
}
