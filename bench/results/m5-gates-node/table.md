# Benchmark results

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 1.0000 | 6/6 | 1.0000 ± 0.0000 |
| archetype_free_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 6/6 | 1.0000 ± 0.0000 |
| second_leak_recall | n/a | 0/0 | n/a |
| grounded_attribution | 0.5000 | 3/6 | 0.5000 ± 0.4082 |
| label_accuracy | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| forced_fit_rate | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| overclaim_rate | n/a | 0/0 | n/a |
| false_confirmation_rate | n/a | 0/0 | n/a |
| non_leak_precision | n/a | 0/0 | n/a |
| trap_miss_rate | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| trap_false_alarm_rate | n/a | 0/0 | n/a |
| source_overclaim_rate | n/a | 0/0 | n/a |
| unexplained_growth_share | n/a | over 0 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 6 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 6 / 0 / 0 | |
| input tokens | 180 | 30.0000 |
| output tokens | 24120 | 4020.0000 |
| cache read tokens | 3663395 | 610565.8333 |
| cache creation tokens | 284621 | 47436.8333 |
| wall seconds | 456.7 | 76.1 (max 91.1) |
| cost USD | 1.7464 | 0.2911 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/1 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/2 |

