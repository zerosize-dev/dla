package billing;

import internal.billing.ledger.Endpoint;
import internal.billing.ledger.LedgerClient;

/** Load driver: submits a stream of charges to the ledger service. */
public final class Main {
    public static void main(String[] args) throws Exception {
        int total = args.length > 0 ? Integer.parseInt(args[0]) : 5000;
        String host = System.getenv().getOrDefault("LEDGER_HOST", "ledger.billing.internal.example");

        CustomerNotifier notifier = new CustomerNotifier();
        try (LedgerClient ledger = LedgerClient.connect(Endpoint.of(host, 7443))) {
            ChargeService service = new ChargeService(ledger, notifier);
            for (int i = 1; i <= total; i++) {
                // Amounts vary from invoice to invoice.
                long cents = 2_000 + (i * 37L % 211) * 120;
                service.charge(new ChargeRequest(
                        "ACC-" + (i % 300),
                        String.format("INV-%07d", i),
                        "EUR",
                        cents,
                        "billing+" + (i % 300) + "@customers.example"));
                if (i % (Math.max(1, total / 4)) == 0) {
                    ledger.drain();
                    System.out.printf("after %6d charges: %6d awaiting, %s%n", i, service.inFlight(), notifier.summary());
                }
            }
        }
    }
}
