package shop;

/** One call from a client asking to place an order. */
record OrderRequest(String customer, String requestId, int basketSeed, String notes) {

    /** Orders are identified by who asked and which request carried the ask. */
    String orderId() {
        return "ord-" + customer + "-" + requestId;
    }
}
