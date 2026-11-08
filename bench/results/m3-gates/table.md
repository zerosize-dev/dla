# Benchmark results

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.9667 | 29/30 | 0.9667 ± 0.0471 |
| archetype_free_recall | 1.0000 | 9/9 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 30/30 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.7931 | 23/29 | 0.7926 ± 0.0105 |
| label_accuracy | 0.0000 | 0/20 | 0.0000 ± 0.0000 |
| forced_fit_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| overclaim_rate | 0.0000 | 0/6 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0000 | 0/27 | 0.0000 ± 0.0000 |
| non_leak_precision | 1.0000 | 12/12 | 1.0000 ± 0.0000 |
| trap_miss_rate | 0.0667 | 1/15 | 0.0667 ± 0.0943 |
| trap_false_alarm_rate | 0.0000 | 0/15 | 0.0000 ± 0.0000 |
| unexplained_growth_share | 0.5197 | over 8 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 60 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 60 / 0 / 0 | |
| input tokens | 1806 | 30.1000 |
| output tokens | 247425 | 4123.7500 |
| cache read tokens | 30701290 | 511688.1667 |
| cache creation tokens | 2083863 | 34731.0500 |
| wall seconds | 11124.8 | 185.4 (max 876.5) |
| cost USD | 13.8834 | 0.2314 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| go-subscription-workers | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 3 | - |
| jvm-audit-chain | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-order-cache | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 3 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| native-glyph-cache | non-leak | 3 | 0 | 3 | - |
| native-vendor-codec | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/0 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/0 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 0 | 3 | - |
| python-graph-churn | non-leak | 3 | 0 | 3 | - |
| python-ingest-two-leaks | multi-leak | 3 | 3 | 3 | L1: 3/3/3; L2: 3/3/3 |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 3 | - |
| rust-batch-buffers | non-leak | 3 | 0 | 3 | - |
| rust-tree-parent-links | archetype-leak | 3 | 3 | 3 | L1: 2/3/2 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 3 | - |

