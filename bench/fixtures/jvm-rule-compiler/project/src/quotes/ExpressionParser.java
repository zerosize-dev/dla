package quotes;

/** Parses expressions such as {@code (qty * unit - 250) * (100 - 12) / 100 + 340}. */
final class ExpressionParser {
    private final String text;
    private int position;

    private ExpressionParser(String text) {
        this.text = text;
    }

    static Node parse(String text) {
        ExpressionParser parser = new ExpressionParser(text);
        Node node = parser.sum();
        parser.skipSpaces();
        if (parser.position != text.length()) {
            throw new IllegalArgumentException("unexpected '" + text.charAt(parser.position) + "' at " + parser.position);
        }
        return node;
    }

    private Node sum() {
        Node left = product();
        while (true) {
            skipSpaces();
            char c = peek();
            if (c != '+' && c != '-') {
                return left;
            }
            position++;
            left = new Node.Binary(c, left, product());
        }
    }

    private Node product() {
        Node left = atom();
        while (true) {
            skipSpaces();
            char c = peek();
            if (c != '*' && c != '/') {
                return left;
            }
            position++;
            left = new Node.Binary(c, left, atom());
        }
    }

    private Node atom() {
        skipSpaces();
        char c = peek();
        if (c == '(') {
            position++;
            Node inner = sum();
            skipSpaces();
            if (peek() != ')') {
                throw new IllegalArgumentException("missing ')' at " + position);
            }
            position++;
            return inner;
        }
        if (Character.isDigit(c)) {
            int start = position;
            while (Character.isDigit(peek())) {
                position++;
            }
            return new Node.Number(Integer.parseInt(text.substring(start, position)));
        }
        if (text.startsWith("qty", position)) {
            position += 3;
            return new Node.Variable(1);
        }
        if (text.startsWith("unit", position)) {
            position += 4;
            return new Node.Variable(2);
        }
        throw new IllegalArgumentException("unexpected '" + c + "' at " + position);
    }

    private char peek() {
        return position < text.length() ? text.charAt(position) : '\0';
    }

    private void skipSpaces() {
        while (peek() == ' ') {
            position++;
        }
    }
}
