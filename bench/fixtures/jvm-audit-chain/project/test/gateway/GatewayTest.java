package gateway;

import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;

/** Plain assertions; run with -ea. */
public final class GatewayTest {
    public static void main(String[] args) {
        balanceAndTransfer();
        unknownPathIsRejected();
        auditLineMentionsPreviousRequest();
        System.out.println("all tests passed");
    }

    private static byte[] bytes(String text) {
        return text.getBytes(StandardCharsets.UTF_8);
    }

    private static Gateway gateway(List<String> audit) {
        return new Gateway(new Handlers(new Ledger(4, 1_000)), audit::add);
    }

    private static void balanceAndTransfer() {
        List<String> audit = new ArrayList<>();
        Gateway gateway = gateway(audit);
        assert gateway.handle("a", "/balance", bytes("AC-0001"), 10).text().equals("balance 1000");
        assert gateway.handle("a", "/transfer", bytes("AC-0001,AC-0002,250"), 20).status() == 200;
        assert gateway.handle("a", "/balance", bytes("AC-0002"), 30).text().equals("balance 1250");
        assert gateway.handle("a", "/transfer", bytes("AC-0001,AC-0002,9999"), 40).status() == 409;
        assert gateway.handle("a", "/statement", bytes("AC-0001"), 50).text().contains("-250 to AC-0002");
    }

    private static void unknownPathIsRejected() {
        Gateway gateway = gateway(new ArrayList<>());
        assert gateway.handle("a", "/export", bytes("AC-0001"), 10).status() == 404;
        assert gateway.handle("a", "/balance", bytes("AC-9999"), 20).status() == 404;
    }

    private static void auditLineMentionsPreviousRequest() {
        List<String> audit = new ArrayList<>();
        Gateway gateway = gateway(audit);
        gateway.handle("a", "/balance", bytes("AC-0001"), 1_000_000);
        gateway.handle("b", "/statement", bytes("AC-0002"), 3_000_000);
        String last = audit.get(audit.size() - 1);
        assert last.contains("(after /balance, +2000us)") : last;
        assert gateway.served() == 2;
    }
}
