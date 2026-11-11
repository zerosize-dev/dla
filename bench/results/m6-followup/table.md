# Benchmark results

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 1.0000 | 24/24 | 1.0000 ± 0.0000 |
| archetype_free_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 24/24 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.9167 | 22/24 | 0.9167 ± 0.0589 |
| label_accuracy | 0.8095 | 17/21 | 0.8095 ± 0.0673 |
| forced_fit_rate | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| overclaim_rate | n/a | 0/0 | n/a |
| false_confirmation_rate | 0.0556 | 1/18 | 0.0556 ± 0.0786 |
| non_leak_precision | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| trap_miss_rate | 0.0000 | 0/15 | 0.0000 ± 0.0000 |
| trap_false_alarm_rate | 0.0667 | 1/15 | 0.0667 ± 0.0943 |
| source_overclaim_rate | n/a | 0/0 | n/a |
| unexplained_growth_share | 1.0195 | over 33 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 39 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 39 / 0 / 0 | |
| input tokens | 2210 | 56.6667 |
| output tokens | 298434 | 7652.1538 |
| cache read tokens | 54168400 | 1388933.3333 |
| cache creation tokens | 2195869 | 56304.3333 |
| wall seconds | 7313.0 | 187.5 (max 415.5) |
| cost USD | 18.6987 | 0.4795 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| jni-order-codec | multi-leak | 3 | 3 | 3 | L1: 3/3/3 (workers jvm 3); L2: 3/3/3 (workers jvm 3) |
| jvm-extension-reload | archetype-leak | 3 | 3 | 3 | L1: 3/3/1 |
| jvm-extension-reload-stopped | decoy-control | 3 | 0 | 3 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| native-frame-decoder-freed | decoy-control | 3 | 0 | 3 | - |
| node-price-board | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| node-price-board-flat | decoy-control | 3 | 0 | 2 | - |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 1 | 3 | - |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 3 | - |

