package gateway;

import java.util.HashMap;
import java.util.Map;
import java.util.function.Consumer;

/** Routes requests to handlers and writes one audit line per request. */
final class Gateway {
    private final Map<String, Handlers.Handler> routes = new HashMap<>();
    private final Consumer<String> auditSink;
    private int served;

    Gateway(Handlers handlers, Consumer<String> auditSink) {
        routes.put("/balance", handlers.balance());
        routes.put("/transfer", handlers.transfer());
        routes.put("/statement", handlers.statement());
        this.auditSink = auditSink;
    }

    Response handle(String caller, String path, byte[] body, long nowNanos) {
        RequestContext context = RequestContext.open(caller, path, body, nowNanos);
        Handlers.Handler handler = routes.get(path);
        Response response = handler == null
                ? Response.rejected(404, "unknown path")
                : handler.handle(context);
        served++;
        auditSink.accept(auditLine(context, response));
        return response;
    }

    int served() {
        return served;
    }

    private static String auditLine(RequestContext context, Response response) {
        return "#" + context.sequence() + " " + context.caller() + " " + context.path()
                + " -> " + response.status()
                + " (after " + context.previousPath() + ", +" + context.microsSincePrevious() + "us)";
    }
}
