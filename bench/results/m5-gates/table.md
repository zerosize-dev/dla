wrote bench/results/m5-gates/table.md
pes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 1.0000 | 39/39 | 1.0000 ± 0.0000 |
| archetype_free_recall | 1.0000 | 9/9 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 39/39 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 6/6 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.8462 | 33/39 | 0.8462 ± 0.0000 |
| label_accuracy | 0.0000 | 0/30 | 0.0000 ± 0.0000 |
| forced_fit_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| overclaim_rate | 0.0000 | 0/12 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0000 | 0/30 | 0.0000 ± 0.0000 |
| non_leak_precision | 1.0000 | 12/12 | 1.0000 ± 0.0000 |
| trap_miss_rate | 0.0000 | 0/18 | 0.0000 ± 0.0000 |
| trap_false_alarm_rate | 0.0000 | 0/18 | 0.0000 ± 0.0000 |
| source_overclaim_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| unexplained_growth_share | 1.0000 | over 2 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 75 | |
| errored runs | 0 | |
| findings source (ledger / findings.json / none) | 70 / 0 / 5 | |
| input tokens | 1938 | 25.8400 |
| output tokens | 243049 | 3240.6533 |
| cache read tokens | 36085199 | 481135.9867 |
| cache creation tokens | 2898891 | 38651.8800 |
| wall seconds | 13726.8 | 183.0 (max 1318.6) |
| cost USD | 18.6647 | 0.2489 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| go-subscription-workers | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 3 | - |
| jni-order-codec | multi-leak | 3 | 3 | 3 | L1: 3/3/3 (workers jvm 3); L2: 3/3/3 (workers jvm 3) |
| jvm-audit-chain | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-order-cache | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 3 | - |
| jvm-rfq-dump-series | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-rfq-dump-series-capped | decoy-control | 3 | 0 | 1 | - |
| jvm-rfq-single-dump | impossible | 3 | 0 | 0 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| native-glyph-cache | non-leak | 3 | 0 | 3 | - |
| native-vendor-codec | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/0 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/0 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 0 | 2 | - |
| python-graph-churn | non-leak | 3 | 0 | 3 | - |
| python-ingest-two-leaks | multi-leak | 3 | 3 | 3 | L1: 3/3/3; L2: 3/3/3 |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 3 | - |
| rust-batch-buffers | non-leak | 3 | 0 | 3 | - |
| rust-tree-parent-links | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 3 | - |
| symptom-no-target | impossible | 3 | 0 | 0 | - |

