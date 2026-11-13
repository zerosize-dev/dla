# jvm-rule-compiler

Category `archetype-free`. Cause `UNCLASSIFIED`, holder `static/global`, region
`non-heap` (metaspace; the class mirrors also use some heap).

## Mechanism

Each customer has pricing terms written as an expression over `qty` and `unit`
(`CustomerTerms.expressionFor`). `PricingRules.forExpression` compiles an
expression to a class implementing `Rule` and defines it in one `RuleLoader`,
held by a static field (`PricingRules.java:7`). The class name is a hash of the
expression text, so a customer who comes back reuses the class
(`RuleLoader.definition` asks `findLoadedClass` first). `defineClass` is at
`src/quotes/RuleLoader.java:17`. A class stays loaded while its loader is
alive, so every distinct expression adds a class to the loader and nothing ever
removes one. The rule instances themselves are garbage after the quote; no
collection of rules or references exists anywhere.

Expected site: `src/quotes/RuleLoader.java:17`. Alternatives: the static loader
field (`PricingRules.java:7`) and the call that defines on demand (`:13`).

## Why `UNCLASSIFIED`

- Not `L-COLLECT`: there is no `Map`, `List` or `Set` that grows. The registry
  of the classes is the loader itself (`findLoadedClass`).
- Not `L-UNLOAD`: nothing reloads. There is one loader for the life of the
  process and no module is redeployed; no earlier loader is kept by a registry.
  The loader is held by a static field on purpose, and the classes could only
  be reclaimed by dropping the loader, which the program never does. A reader
  who sees a class loader and growing loaded-class counts may reach for
  `L-UNLOAD`: its fingerprint is a class loader per reload, held by a registry,
  and that is not what happens here.
- Not `L-REGISTER` or `L-CAPTURE`: no listener, timer or closure is involved.

The growth is runtime bookkeeping (classes and their metadata), proportional to
the number of distinct customers.

## Measurements

Toolchain: OpenJDK 21.0.12.

```
cd project && ./build.sh run 8000        # compiles, runs the unit tests, prices 8000 requests
```

The driver forces a GC, then prints loaded classes (`ClassLoadingMXBean`),
metaspace in use (the `Metaspace` memory pool) and heap in use. Last checkpoint
of three separate runs (7 of 8 requests come from a customer not seen before):

| quotes | classes loaded | metaspace | heap in use after GC |
|---|---|---|---|
| 2000  | 2818  | 3731 KB  | 2690 KB |
| 8000  | 8068  | 10759 KB | 5639 KB |
| 32000 | 29068 | 38886 KB | 17941 KB |

Between 8000 and 32000 quotes: 0.875 classes, 1.17 KB of metaspace and 0.51 KB of
heap per quote, linear in the iteration count. A 150000-quote run under `-Xint`
ends at 132302 classes and 164153 KB of metaspace; no plateau. Unit tests: 6
in `test/quotes/QuoteServiceTest.java`, all pass.
