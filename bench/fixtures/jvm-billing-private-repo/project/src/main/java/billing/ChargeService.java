package billing;

import internal.billing.ledger.LedgerClient;
import internal.billing.ledger.Money;
import internal.billing.ledger.Posting;
import internal.billing.ledger.Ticket;

/** Turns charge requests into ledger postings. */
public final class ChargeService {
    private final LedgerClient ledger;
    private final CustomerNotifier notifier;

    public ChargeService(LedgerClient ledger, CustomerNotifier notifier) {
        this.ledger = ledger;
        this.notifier = notifier;
    }

    public String charge(ChargeRequest request) {
        Posting posting = Posting.builder()
                .account(request.accountId())
                .amount(Money.ofCents(request.currency(), request.cents()))
                .reference(request.invoiceNumber())
                .build();
        PendingCharge pending = new PendingCharge(request, System.nanoTime());
        Ticket ticket = ledger.newTicket();
        InFlightCharges.register(ticket.id(), pending);
        ledger.submit(ticket, posting, new ChargeListener(pending, notifier));
        return ticket.id();
    }

    public int inFlight() {
        return InFlightCharges.count();
    }
}
