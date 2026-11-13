package quotes;

import java.nio.charset.StandardCharsets;

/** Turns a customer's pricing expression into a {@link Rule}, compiling each expression once. */
final class PricingRules {
    private static final RuleLoader LOADER = new RuleLoader(PricingRules.class.getClassLoader());

    private PricingRules() {}

    static Rule forExpression(String expression) {
        String name = "quotes.gen.Rule" + Long.toHexString(fingerprint(expression));
        Class<?> type = LOADER.definition(name, () -> ClassWriter.write(name.replace('.', '/'), ExpressionParser.parse(expression)));
        try {
            return (Rule) type.getDeclaredConstructor().newInstance();
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException("cannot create " + name, e);
        }
    }

    /** 64-bit FNV-1a of the expression text. */
    private static long fingerprint(String expression) {
        long hash = 0xcbf29ce484222325L;
        for (byte b : expression.getBytes(StandardCharsets.UTF_8)) {
            hash ^= (b & 0xFF);
            hash *= 0x100000001b3L;
        }
        return hash;
    }
}
