package quotes;

/** Plain assertions; run with -ea. */
public final class QuoteServiceTest {
    public static void main(String[] args) {
        parserKeepsPrecedence();
        parserRejectsGarbage();
        compiledRuleMatchesTheExpression();
        sameExpressionReusesItsRule();
        customersGetTheirOwnTerms();
        serviceTotalsQuotes();
        System.out.println("all tests passed");
    }

    private static void parserKeepsPrecedence() {
        Rule rule = PricingRules.forExpression("2 + qty * 3 - (unit - 4) / 2");
        assert rule.apply(5, 10) == 2 + 5 * 3 - (10 - 4) / 2 : rule.apply(5, 10);
    }

    private static void parserRejectsGarbage() {
        try {
            ExpressionParser.parse("qty * (unit + 3");
            assert false : "expected a rejection";
        } catch (IllegalArgumentException expected) {
            assert expected.getMessage().contains("')'");
        }
        try {
            ExpressionParser.parse("qty $ 3");
            assert false : "expected a rejection";
        } catch (IllegalArgumentException expected) {
            assert expected.getMessage().contains("'$'");
        }
    }

    private static void compiledRuleMatchesTheExpression() {
        Rule rule = PricingRules.forExpression("(qty * unit - 250) * (100 - 12) / 100 + 340");
        assert rule.apply(3, 1000) == (3 * 1000 - 250) * (100 - 12) / 100 + 340;
        Rule wide = PricingRules.forExpression("qty * 70000 + 123456");
        assert wide.apply(2, 0) == 2 * 70000 + 123456;
    }

    private static void sameExpressionReusesItsRule() {
        Rule a = PricingRules.forExpression("qty * unit + 1");
        Rule b = PricingRules.forExpression("qty * unit + 1");
        Rule c = PricingRules.forExpression("qty * unit + 2");
        assert a.getClass() == b.getClass();
        assert a.getClass() != c.getClass();
    }

    private static void customersGetTheirOwnTerms() {
        assert !CustomerTerms.expressionFor(1).equals(CustomerTerms.expressionFor(2));
        assert CustomerTerms.expressionFor(7).equals(CustomerTerms.expressionFor(7));
    }

    private static void serviceTotalsQuotes() {
        QuoteService service = new QuoteService();
        int first = service.quote(11, 4, 500);
        int second = service.quote(11, 4, 500);
        assert first == second;
        assert service.quoted() == 2 && service.totalCents() == 2L * first;
    }
}
