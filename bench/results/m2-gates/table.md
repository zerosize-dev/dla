# Benchmark results

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.0000 | 0/30 | 0.0000 ± 0.0000 |
| archetype_free_recall | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 30/30 | 1.0000 ± 0.0000 |
| second_leak_recall | n/a | 0/0 | n/a |
| grounded_attribution | n/a | 0/0 | n/a |
| label_accuracy | n/a | 0/0 | n/a |
| forced_fit_rate | n/a | 0/0 | n/a |
| overclaim_rate | 0.0000 | 0/6 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0000 | 0/27 | 0.0000 ± 0.0000 |
| non_leak_precision | 0.0000 | 0/12 | 0.0000 ± 0.0000 |
| trap_miss_rate | 1.0000 | 15/15 | 1.0000 ± 0.0000 |
| trap_false_alarm_rate | 0.0000 | 0/15 | 0.0000 ± 0.0000 |
| unexplained_growth_share | n/a | over 0 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 60 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 60 / 0 / 0 | |
| input tokens | 1238 | 20.6333 |
| output tokens | 203694 | 3394.9000 |
| cache read tokens | 18201162 | 303352.7000 |
| cache creation tokens | 1653011 | 27550.1833 |
| wall seconds | 3824.6 | 63.7 (max 798.7) |
| cost USD | 10.7688 | 0.1795 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 0 | - |
| go-subscription-workers | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 0 | - |
| jvm-audit-chain | archetype-free | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-order-cache | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 0 | - |
| native-frame-decoder | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| native-glyph-cache | non-leak | 3 | 0 | 0 | - |
| native-vendor-codec | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| node-retry-scheduler | archetype-free | 3 | 0 | 0 | L1: 0/3/0 |
| node-socket-heartbeat | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 0 | 0 | - |
| python-graph-churn | non-leak | 3 | 0 | 0 | - |
| python-ingest-two-leaks | multi-leak | 3 | 0 | 0 | L1: 0/3/0; L2: 0/3/0 |
| python-report-memo | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 0 | - |
| rust-batch-buffers | non-leak | 3 | 0 | 0 | - |
| rust-tree-parent-links | archetype-leak | 3 | 0 | 0 | L1: 0/3/0 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 0 | - |

