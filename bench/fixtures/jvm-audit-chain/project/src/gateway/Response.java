package gateway;

/** What the gateway sends back. */
record Response(int status, String text) {
    static Response ok(String text) {
        return new Response(200, text);
    }

    static Response rejected(int status, String reason) {
        return new Response(status, reason);
    }
}
