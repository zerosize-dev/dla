package portal;

/** The user accounts the portal knows about; a stand-in for the identity service. */
final class Directory {
    private static final String[] DEVICES = {"laptop", "phone", "tablet", "kiosk", "desktop"};

    private final int users;

    Directory(int users) {
        this.users = users;
    }

    String userAt(int seed) {
        return "user-" + Math.floorMod(seed * 31 + 7, users);
    }

    String deviceAt(int seed) {
        return DEVICES[Math.floorMod(seed, DEVICES.length)];
    }

    /** Accounts under review by the support team; the operator signs them out now and then. */
    String watchedUser(int round) {
        return "user-" + Math.floorMod(round, 12);
    }

    boolean exists(String userId) {
        if (!userId.startsWith("user-")) {
            return false;
        }
        try {
            int number = Integer.parseInt(userId.substring(5));
            return number >= 0 && number < users;
        } catch (NumberFormatException e) {
            return false;
        }
    }
}
