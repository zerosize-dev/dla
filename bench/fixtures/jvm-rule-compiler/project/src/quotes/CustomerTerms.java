package quotes;

/** The negotiated pricing terms of a customer, written as an expression over qty and unit. */
final class CustomerTerms {
    private CustomerTerms() {}

    /** Every customer has a rebate on the list price, a percentage off, and a handling fee. */
    static String expressionFor(long customerId) {
        long mixed = customerId * 0x9E3779B97F4A7C15L;
        int rebate = (int) ((mixed >>> 20) % 1000);
        int percentOff = (int) ((mixed >>> 33) % 50);
        int handling = (int) ((mixed >>> 41) % 5000);
        return "(qty * unit - " + rebate + ") * (100 - " + percentOff + ") / 100 + " + handling;
    }
}
