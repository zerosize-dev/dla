# Benchmark results

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 1.0000 | 21/21 | 1.0000 ± 0.0000 |
| archetype_free_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 21/21 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.8571 | 18/21 | 0.8571 ± 0.0000 |
| label_accuracy | 0.7222 | 13/18 | 0.7222 ± 0.0786 |
| forced_fit_rate | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| overclaim_rate | n/a | 0/0 | n/a |
| false_confirmation_rate | 0.0000 | 0/6 | 0.0000 ± 0.0000 |
| non_leak_precision | n/a | 0/0 | n/a |
| trap_miss_rate | 0.0000 | 0/12 | 0.0000 ± 0.0000 |
| trap_false_alarm_rate | 0.0000 | 0/6 | 0.0000 ± 0.0000 |
| source_overclaim_rate | n/a | 0/0 | n/a |
| unexplained_growth_share | 0.9961 | over 21 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 24 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 24 / 0 / 0 | |
| input tokens | 1478 | 61.5833 |
| output tokens | 197124 | 8213.5000 |
| cache read tokens | 36694926 | 1528955.2500 |
| cache creation tokens | 1412427 | 58851.1250 |
| wall seconds | 5171.5 | 215.5 (max 472.4) |
| cost USD | 12.2243 | 0.5093 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| jni-order-codec | multi-leak | 3 | 3 | 3 | L1: 3/3/3 (workers jvm 2); L2: 3/3/3 (workers jvm 2) |
| jvm-extension-reload | archetype-leak | 3 | 3 | 3 | L1: 3/3/1 |
| node-price-board | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| node-price-board-flat | decoy-control | 3 | 0 | 2 | - |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/2 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 0 | 3 | - |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |

