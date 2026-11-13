package quotes;

/** Syntax tree of a pricing expression. */
sealed interface Node {
    record Number(int value) implements Node {}

    /** Variable 0 is the quantity, variable 1 the unit price. */
    record Variable(int slot) implements Node {}

    record Binary(char operator, Node left, Node right) implements Node {}

    /** Operand stack slots needed to evaluate the node. */
    static int depth(Node node) {
        return switch (node) {
            case Number n -> 1;
            case Variable v -> 1;
            case Binary b -> Math.max(depth(b.left()), 1 + depth(b.right()));
        };
    }
}
