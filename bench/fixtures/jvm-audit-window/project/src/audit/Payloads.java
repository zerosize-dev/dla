package audit;

/** Builds the JSON-like request payload stored with an entry. */
final class Payloads {
    private static final String[] FIELDS = {"plan", "seats", "region", "contact", "limit", "label"};

    private Payloads() {}

    static String describe(int seed, String action) {
        StringBuilder text = new StringBuilder("{\"action\":\"").append(action).append("\",\"request\":{");
        for (int i = 0; i < FIELDS.length; i++) {
            if (i > 0) {
                text.append(',');
            }
            text.append('"').append(FIELDS[i]).append("\":\"")
                    .append(Integer.toString((seed + i) * 2654435, 36)).append("-").append((seed + i) % 97).append('"');
        }
        return text.append("},\"client\":\"console/").append(seed % 13).append("\"}").toString();
    }
}
