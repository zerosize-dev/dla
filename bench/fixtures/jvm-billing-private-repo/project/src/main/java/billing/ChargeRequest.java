package billing;

/** A request to collect money for one invoice. */
public record ChargeRequest(String accountId, String invoiceNumber, String currency, long cents, String customerEmail) {

    public ChargeRequest {
        if (cents <= 0) {
            throw new IllegalArgumentException("amount must be positive: " + cents);
        }
    }
}
