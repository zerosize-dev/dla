package gateway;

import java.nio.charset.StandardCharsets;

/** The three endpoints the gateway serves. */
final class Handlers {
    interface Handler {
        Response handle(RequestContext context);
    }

    private final Ledger ledger;

    Handlers(Ledger ledger) {
        this.ledger = ledger;
    }

    Handler balance() {
        return context -> {
            Long cents = ledger.balance(text(context));
            return cents == null ? Response.rejected(404, "no such account") : Response.ok("balance " + cents);
        };
    }

    Handler transfer() {
        return context -> {
            String[] parts = text(context).split(",");
            if (parts.length != 3) {
                return Response.rejected(400, "expected from,to,cents");
            }
            long cents;
            try {
                cents = Long.parseLong(parts[2].trim());
            } catch (NumberFormatException e) {
                return Response.rejected(400, "amount is not a number");
            }
            return ledger.transfer(parts[0].trim(), parts[1].trim(), cents)
                    ? Response.ok("moved " + cents)
                    : Response.rejected(409, "transfer refused");
        };
    }

    Handler statement() {
        return context -> {
            String lines = ledger.statement(text(context));
            return lines == null ? Response.rejected(404, "no such account") : Response.ok(lines);
        };
    }

    private static String text(RequestContext context) {
        return new String(context.body(), StandardCharsets.UTF_8).trim();
    }
}
