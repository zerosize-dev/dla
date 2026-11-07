package billing;

/** What the service remembers about a charge between submitting it and hearing back. */
record PendingCharge(ChargeRequest request, long submittedAtNanos) {

    long ageMillis(long nowNanos) {
        return (nowNanos - submittedAtNanos) / 1_000_000;
    }
}
