# Benchmark results

## no-skill (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.3667 | 11/30 | 0.3667 ± 0.0943 |
| archetype_free_recall | 0.4444 | 4/9 | 0.4444 ± 0.1571 |
| flagged_recall | 0.3667 | 11/30 | 0.3667 ± 0.0943 |
| second_leak_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| grounded_attribution | 1.0000 | 11/11 | 1.0000 ± 0.0000 |
| label_accuracy | n/a | 0/0 | n/a |
| forced_fit_rate | n/a | 0/0 | n/a |
| overclaim_rate | 0.0000 | 0/6 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.1481 | 4/27 | 0.1481 ± 0.0524 |
| non_leak_precision | 0.7500 | 9/12 | 0.7500 ± 0.0000 |
| trap_miss_rate | 0.8000 | 12/15 | 0.8000 ± 0.1633 |
| trap_false_alarm_rate | 0.0667 | 1/15 | 0.0667 ± 0.0943 |
| unexplained_growth_share | n/a | over 0 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 60 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 0 / 60 / 0 | |
| input tokens | 694 | 11.5667 |
| output tokens | 131828 | 2197.1333 |
| cache read tokens | 8041747 | 134029.1167 |
| cache creation tokens | 852298 | 14204.9667 |
| wall seconds | 2325.0 | 38.7 (max 84.7) |
| cost USD | 5.5330 | 0.0922 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| go-subscription-workers | archetype-leak | 3 | 3 | 3 | L1: 1/1/1 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 3 | - |
| jvm-audit-chain | archetype-free | 3 | 3 | 3 | L1: 1/1/1 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/2/0 |
| jvm-order-cache | archetype-leak | 3 | 3 | 1 | L1: 1/1/1 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 3 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 1/1/1 |
| native-glyph-cache | non-leak | 3 | 0 | 3 | - |
| native-vendor-codec | impossible | 3 | 0 | 2 | L1: 0/2/0 |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 0/0/0 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 1/1/1 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 1 | 3 | - |
| python-graph-churn | non-leak | 3 | 0 | 3 | - |
| python-ingest-two-leaks | multi-leak | 3 | 3 | 2 | L1: 3/3/3; L2: 3/3/3 |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 0/0/0 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 3 | - |
| rust-batch-buffers | non-leak | 3 | 3 | 3 | - |
| rust-tree-parent-links | archetype-leak | 3 | 3 | 2 | L1: 0/0/0 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 3 | - |

