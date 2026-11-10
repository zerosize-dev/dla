# Benchmark results

## full (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.9412 | 16/17 | 0.9412 ± 0.0000 |
| archetype_free_recall | 1.0000 | 3/3 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 17/17 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 2/2 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.9375 | 15/16 | 0.9375 ± 0.0000 |
| label_accuracy | 0.8462 | 11/13 | 0.8462 ± 0.0000 |
| forced_fit_rate | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| overclaim_rate | 0.0000 | 0/4 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0000 | 0/15 | 0.0000 ± 0.0000 |
| non_leak_precision | 0.7500 | 3/4 | 0.7500 ± 0.0000 |
| trap_miss_rate | 0.0909 | 1/11 | 0.0909 ± 0.0000 |
| trap_false_alarm_rate | 0.0000 | 0/11 | 0.0000 ± 0.0000 |
| source_overclaim_rate | 0.0000 | 0/3 | 0.0000 ± 0.0000 |
| unexplained_growth_share | 0.9916 | over 24 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 34 | |
| errored runs | 1 | |
| findings source (ledger / findings.json / none) | 33 / 0 / 1 | |
| input tokens | 1622 | 47.7059 |
| output tokens | 216987 | 6381.9706 |
| cache read tokens | 38757662 | 1139931.2353 |
| cache creation tokens | 1711732 | 50345.0588 |
| wall seconds | 9424.5 | 277.2 (max 1500.1) |
| cost USD | 14.5094 | 0.4397 |

### Verification

| Item | Value | Num/Den |
|---|---|---|
| verified share of confirmed leak-side findings | 1.0000 | 22/22 |
| site agreement (verified findings) | 0.3182 | 7/22 |
| holder agreement (verified findings) | 0.9091 | 20/22 |
| label agreement (verified findings with a label verdict) | 0.6818 | 15/22 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 1 | 0 | 0 | - |
| go-subscription-workers | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| go-subscription-workers-closed | decoy-control | 1 | 0 | 1 | - |
| jni-order-codec | multi-leak | 1 | 1 | 1 | L1: 1/1/1 (workers jvm 1); L2: 1/1/1 (workers jvm 1) |
| jvm-audit-chain | archetype-free | 1 | 1 | 1 | L1: 1/1/1 |
| jvm-billing-private-repo | impossible | 1 | 0 | 0 | L1: 0/1/0 |
| jvm-extension-reload | archetype-leak | 1 | 1 | 1 | L1: 1/1/0 |
| jvm-extension-reload-stopped | decoy-control | 1 | 0 | 1 | - |
| jvm-invoice-render | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| jvm-invoice-render-trimmed | decoy-control | 1 | 0 | 1 | - |
| jvm-order-cache | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| jvm-order-cache-bounded | decoy-control | 1 | 0 | 1 | - |
| jvm-rfq-dump-series | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| jvm-rfq-dump-series-capped | decoy-control | 1 | 0 | 0 | - |
| jvm-rfq-single-dump | impossible | 1 | 0 | 0 | - |
| native-frame-decoder | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| native-frame-decoder-freed | decoy-control | 1 | 0 | 1 | - |
| native-glyph-cache | non-leak | 1 | 0 | 1 | - |
| native-vendor-codec | impossible | 1 | 0 | 0 | L1: 0/1/0 |
| node-price-board | archetype-leak | 1 | 1 | 1 | L1: 0/1/0 |
| node-price-board-flat | decoy-control | 1 | 0 | 0 | - |
| node-retry-scheduler | archetype-free | 1 | 1 | 1 | L1: 1/1/1 |
| node-socket-heartbeat | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| node-socket-heartbeat-cleared | decoy-control | 1 | 0 | 1 | - |
| python-graph-churn | non-leak | 1 | 0 | 1 | - |
| python-ingest-two-leaks | multi-leak | 1 | 1 | 1 | L1: 1/1/1; L2: 1/1/1 |
| python-report-memo | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| python-report-memo-bounded | decoy-control | 1 | 0 | 1 | - |
| rust-batch-buffers | non-leak | 1 | 0 | 1 | - |
| rust-meter-callbacks | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| rust-meter-callbacks-reclaimed | decoy-control | 1 | 0 | 1 | - |
| rust-tree-parent-links | archetype-leak | 1 | 1 | 1 | L1: 1/1/1 |
| rust-tree-parent-links-weak | decoy-control | 1 | 0 | 1 | - |
| symptom-no-target | impossible | 1 | 0 | 0 | - |

## gates (archetypes off)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.9412 | 48/51 | 0.9412 ± 0.0000 |
| archetype_free_recall | 1.0000 | 9/9 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 51/51 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 6/6 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.8542 | 41/48 | 0.8542 ± 0.0589 |
| label_accuracy | 0.0000 | 0/39 | 0.0000 ± 0.0000 |
| forced_fit_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| overclaim_rate | 0.0000 | 0/12 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0000 | 0/45 | 0.0000 ± 0.0000 |
| non_leak_precision | 1.0000 | 12/12 | 1.0000 ± 0.0000 |
| trap_miss_rate | 0.0909 | 3/33 | 0.0909 ± 0.0000 |
| trap_false_alarm_rate | 0.0000 | 0/33 | 0.0000 ± 0.0000 |
| source_overclaim_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| unexplained_growth_share | 0.9654 | over 74 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 102 | |
| errored runs | 1 | |
| findings source (ledger / findings.json / none) | 96 / 0 / 6 | |
| input tokens | 4682 | 45.9020 |
| output tokens | 595927 | 5842.4216 |
| cache read tokens | 107367726 | 1052624.7647 |
| cache creation tokens | 4972080 | 48745.8824 |
| wall seconds | 19416.7 | 190.4 (max 1500.0) |
| cost USD | 37.5158 | 0.3714 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| go-subscription-workers | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 3 | - |
| jni-order-codec | multi-leak | 3 | 3 | 3 | L1: 3/3/2 (workers jvm 2); L2: 3/3/3 (workers jvm 2) |
| jvm-audit-chain | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-extension-reload | archetype-leak | 3 | 3 | 3 | L1: 3/3/2 |
| jvm-extension-reload-stopped | decoy-control | 3 | 0 | 3 | - |
| jvm-invoice-render | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-invoice-render-trimmed | decoy-control | 3 | 0 | 1 | - |
| jvm-order-cache | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 3 | - |
| jvm-rfq-dump-series | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-rfq-dump-series-capped | decoy-control | 3 | 0 | 0 | - |
| jvm-rfq-single-dump | impossible | 3 | 0 | 0 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| native-frame-decoder-freed | decoy-control | 3 | 0 | 3 | - |
| native-glyph-cache | non-leak | 3 | 0 | 3 | - |
| native-vendor-codec | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| node-price-board | archetype-leak | 3 | 3 | 3 | L1: 0/3/0 |
| node-price-board-flat | decoy-control | 3 | 0 | 3 | - |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/1 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/0 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 0 | 1 | - |
| python-graph-churn | non-leak | 3 | 0 | 3 | - |
| python-ingest-two-leaks | multi-leak | 3 | 3 | 3 | L1: 3/3/3; L2: 3/3/3 |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 2 | - |
| rust-batch-buffers | non-leak | 3 | 0 | 3 | - |
| rust-meter-callbacks | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| rust-meter-callbacks-reclaimed | decoy-control | 3 | 0 | 3 | - |
| rust-tree-parent-links | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 3 | - |
| symptom-no-target | impossible | 3 | 0 | 0 | - |

## gates (archetypes on)

| Metric | Value | Num/Den | Mean ± stdev over runs |
|---|---|---|---|
| leak_recall | 0.9608 | 49/51 | 0.9608 ± 0.0277 |
| archetype_free_recall | 1.0000 | 9/9 | 1.0000 ± 0.0000 |
| flagged_recall | 1.0000 | 51/51 | 1.0000 ± 0.0000 |
| second_leak_recall | 1.0000 | 6/6 | 1.0000 ± 0.0000 |
| grounded_attribution | 0.9592 | 47/49 | 0.9596 ± 0.0286 |
| label_accuracy | 0.8000 | 32/40 | 0.8004 ± 0.0948 |
| forced_fit_rate | 0.3333 | 3/9 | 0.3333 ± 0.0000 |
| overclaim_rate | 0.0000 | 0/12 | 0.0000 ± 0.0000 |
| false_confirmation_rate | 0.0222 | 1/45 | 0.0222 ± 0.0314 |
| non_leak_precision | 1.0000 | 12/12 | 1.0000 ± 0.0000 |
| trap_miss_rate | 0.0606 | 2/33 | 0.0606 ± 0.0429 |
| trap_false_alarm_rate | 0.0303 | 1/33 | 0.0303 ± 0.0429 |
| source_overclaim_rate | 0.0000 | 0/9 | 0.0000 ± 0.0000 |
| unexplained_growth_share | 1.0002 | over 72 run(s) | |

### Usage

| Item | Sum | Mean per run |
|---|---|---|
| runs | 102 | |
| errored runs | 1 | |
| findings source (ledger / findings.json / none) | 96 / 0 / 6 | |
| input tokens | 4946 | 48.4902 |
| output tokens | 619229 | 6070.8725 |
| cache read tokens | 115541412 | 1132758.9412 |
| cache creation tokens | 5145361 | 50444.7157 |
| wall seconds | 21640.5 | 212.2 (max 1500.1) |
| cost USD | 40.2476 | 0.3946 |

### Per fixture

| Fixture | Category | Runs | Claim runs | Rule-out runs | Leaks (found/flagged/grounded) |
|---|---|---|---|---|---|
| go-buffer-pool | non-leak | 3 | 0 | 3 | - |
| go-subscription-workers | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| go-subscription-workers-closed | decoy-control | 3 | 0 | 3 | - |
| jni-order-codec | multi-leak | 3 | 3 | 3 | L1: 3/3/3 (workers jvm 3); L2: 3/3/3 (workers jvm 3) |
| jvm-audit-chain | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-billing-private-repo | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| jvm-extension-reload | archetype-leak | 3 | 3 | 3 | L1: 3/3/2 |
| jvm-extension-reload-stopped | decoy-control | 3 | 0 | 3 | - |
| jvm-invoice-render | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-invoice-render-trimmed | decoy-control | 3 | 0 | 1 | - |
| jvm-order-cache | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-order-cache-bounded | decoy-control | 3 | 0 | 3 | - |
| jvm-rfq-dump-series | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| jvm-rfq-dump-series-capped | decoy-control | 3 | 0 | 0 | - |
| jvm-rfq-single-dump | impossible | 3 | 0 | 0 | - |
| native-frame-decoder | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| native-frame-decoder-freed | decoy-control | 3 | 0 | 3 | - |
| native-glyph-cache | non-leak | 3 | 0 | 3 | - |
| native-vendor-codec | impossible | 3 | 0 | 0 | L1: 0/3/0 |
| node-price-board | archetype-leak | 3 | 3 | 3 | L1: 1/3/1 |
| node-price-board-flat | decoy-control | 3 | 0 | 3 | - |
| node-retry-scheduler | archetype-free | 3 | 3 | 3 | L1: 3/3/3 |
| node-socket-heartbeat | archetype-leak | 3 | 3 | 3 | L1: 3/3/2 |
| node-socket-heartbeat-cleared | decoy-control | 3 | 1 | 3 | - |
| python-graph-churn | non-leak | 3 | 0 | 3 | - |
| python-ingest-two-leaks | multi-leak | 3 | 3 | 3 | L1: 3/3/3; L2: 3/3/3 |
| python-report-memo | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| python-report-memo-bounded | decoy-control | 3 | 0 | 3 | - |
| rust-batch-buffers | non-leak | 3 | 0 | 3 | - |
| rust-meter-callbacks | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| rust-meter-callbacks-reclaimed | decoy-control | 3 | 0 | 3 | - |
| rust-tree-parent-links | archetype-leak | 3 | 3 | 3 | L1: 3/3/3 |
| rust-tree-parent-links-weak | decoy-control | 3 | 0 | 3 | - |
| symptom-no-target | impossible | 3 | 0 | 0 | - |

