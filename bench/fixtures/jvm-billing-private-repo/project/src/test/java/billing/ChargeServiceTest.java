package billing;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import internal.billing.ledger.LedgerClient;
import org.junit.jupiter.api.Test;

class ChargeServiceTest {
    private static ChargeRequest request(String invoice, long cents) {
        return new ChargeRequest("ACC-1", invoice, "EUR", cents, "a@customers.example");
    }

    @Test
    void approvedChargeIsReported() throws Exception {
        CustomerNotifier notifier = new CustomerNotifier();
        try (LedgerClient ledger = LedgerClient.inMemory()) {
            ChargeService service = new ChargeService(ledger, notifier);
            service.charge(request("INV-1", 2_500));
            ledger.drain();
            assertEquals(1, notifier.paidCount());
        }
    }

    @Test
    void amountMustBePositive() {
        assertThrows(IllegalArgumentException.class, () -> request("INV-2", 0));
    }
}
