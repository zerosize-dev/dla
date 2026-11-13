package quotes;

/** Prices quote requests with the rule of the customer who asked. */
final class QuoteService {
    private long quoted;
    private long totalCents;

    int quote(long customerId, int quantity, int unitCents) {
        Rule rule = PricingRules.forExpression(CustomerTerms.expressionFor(customerId));
        int total = rule.apply(quantity, unitCents);
        quoted++;
        totalCents += total;
        return total;
    }

    long quoted() {
        return quoted;
    }

    long totalCents() {
        return totalCents;
    }
}
