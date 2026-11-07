package billing;

import internal.billing.ledger.PostingListener;
import internal.billing.ledger.Receipt;
import internal.billing.ledger.RejectionReason;
import internal.billing.ledger.Ticket;

/** Receives the ledger's verdict on one charge. */
final class ChargeListener implements PostingListener {
    private final PendingCharge pending;
    private final CustomerNotifier notifier;

    ChargeListener(PendingCharge pending, CustomerNotifier notifier) {
        this.pending = pending;
        this.notifier = notifier;
    }

    @Override
    public void onPosted(Receipt receipt) {
        InFlightCharges.complete(receipt.ticketId());
        notifier.paid(pending.request().customerEmail(), pending.request().invoiceNumber());
    }

    @Override
    public void onRejected(Ticket ticket, RejectionReason reason) {
        notifier.declined(pending.request().customerEmail(), pending.request().invoiceNumber(), reason.code());
    }

    @Override
    public void onExpired(Ticket ticket) {
        notifier.delayed(pending.request().customerEmail(), pending.request().invoiceNumber());
    }
}
