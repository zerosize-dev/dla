package gateway;

import java.util.ArrayDeque;
import java.util.Deque;
import java.util.HashMap;
import java.util.Map;

/** In-memory account balances with a short history per account. */
final class Ledger {
    private static final int HISTORY_PER_ACCOUNT = 20;

    private final Map<String, Long> balances = new HashMap<>();
    private final Map<String, Deque<String>> history = new HashMap<>();

    Ledger(int accounts, long openingCents) {
        for (int i = 0; i < accounts; i++) {
            String id = account(i);
            balances.put(id, openingCents);
            history.put(id, new ArrayDeque<>());
        }
    }

    static String account(int index) {
        return String.format("AC-%04d", index);
    }

    Long balance(String account) {
        return balances.get(account);
    }

    boolean transfer(String from, String to, long cents) {
        Long available = balances.get(from);
        if (available == null || !balances.containsKey(to) || cents <= 0 || available < cents) {
            return false;
        }
        balances.put(from, available - cents);
        balances.merge(to, cents, Long::sum);
        note(from, "-" + cents + " to " + to);
        note(to, "+" + cents + " from " + from);
        return true;
    }

    String statement(String account) {
        Deque<String> lines = history.get(account);
        return lines == null ? null : String.join("; ", lines);
    }

    private void note(String account, String line) {
        Deque<String> lines = history.get(account);
        lines.addLast(line);
        while (lines.size() > HISTORY_PER_ACCOUNT) {
            lines.removeFirst();
        }
    }
}
