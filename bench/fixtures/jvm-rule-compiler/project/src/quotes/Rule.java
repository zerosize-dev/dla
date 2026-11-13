package quotes;

/** A compiled pricing rule: turns a quantity and a unit price into a total, both in cents. */
public interface Rule {
    int apply(int quantity, int unitCents);
}
