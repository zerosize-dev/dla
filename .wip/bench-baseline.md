| fixture | category | outcome | where | failure |
|---|---|---|---|---|
| go-buffer-pool | non-leak | pass | tracked before d1fca56 |  |
| go-feed-merge | multi-leak | fail | new in d1fca56 | leak L2 (internal/indexer/indexer.go:24) is not confirmed |
| go-gc-tuned-batch | non-leak | pass | new in d1fca56 |  |
| go-header-index | archetype-free | fail | new in d1fca56 | leak L1 (internal/edge/index.go:36) is not confirmed |
| go-private-module | impossible | pass | new in d1fca56 |  |
| go-subscription-workers | archetype-leak | pass | tracked before d1fca56 |  |
| go-subscription-workers-closed | decoy-control | pass | tracked before d1fca56 |  |
| go-ticker-legacy | archetype-leak | fail | new in d1fca56 | leak L1 (L-REGISTER) label refused label-evidence-missing [summarize heap-graph: did not run | summarize non-heap via go-goroutine: gate fail, tied none, reason |
| go-ticker-modern | decoy-control | pass | new in d1fca56 |  |
| go-trace-context | archetype-free | fail | new in d1fca56 | leak L1 (internal/rates/poller.go:32) is not confirmed |
| jni-order-codec | multi-leak | pass | tracked before d1fca56 |  |
| jvm-audit-chain | archetype-free | pass | tracked before d1fca56 |  |
| jvm-audit-window | non-leak | pass | new in d1fca56 |  |
| jvm-billing-private-repo | impossible | pass | tracked before d1fca56 |  |
| jvm-extension-reload | archetype-leak | pass | tracked before d1fca56 |  |
| jvm-extension-reload-stopped | decoy-control | pass | tracked before d1fca56 |  |
| jvm-invoice-render | archetype-leak | pass | tracked before d1fca56 |  |
| jvm-invoice-render-trimmed | decoy-control | pass | tracked before d1fca56 |  |
| jvm-order-cache | archetype-leak | pass | tracked before d1fca56 |  |
| jvm-order-cache-bounded | decoy-control | pass | tracked before d1fca56 |  |
| jvm-pool-warmup | non-leak | pass | new in d1fca56 |  |
| jvm-rfq-dump-series | archetype-leak | pass | tracked before d1fca56 |  |
| jvm-rfq-dump-series-capped | decoy-control | pass | tracked before d1fca56 |  |
| jvm-rfq-single-dump | impossible | pass | tracked before d1fca56 |  |
| jvm-rule-compiler | archetype-free | pass | new in d1fca56 |  |
| jvm-session-index | archetype-leak | pass | new in d1fca56 |  |
| jvm-session-index-pruned | decoy-control | pass | new in d1fca56 |  |
| native-arena-reuse | non-leak | pass | new in d1fca56 |  |
| native-frame-decoder | archetype-leak | pass | tracked before d1fca56 |  |
| native-frame-decoder-freed | decoy-control | pass | tracked before d1fca56 |  |
| native-glyph-cache | non-leak | pass | tracked before d1fca56 |  |
| native-parcel-router | multi-leak | pass | new in d1fca56 |  |
| native-split-freelist | archetype-free | pass | new in d1fca56 |  |
| native-tombstone-table | archetype-free | pass | new in d1fca56 |  |
| native-vendor-codec | impossible | pass | tracked before d1fca56 |  |
| node-gateway-merge | multi-leak | pass | new in d1fca56 |  |
| node-native-addon | impossible | pass | new in d1fca56 |  |
| node-price-board | archetype-leak | pass | tracked before d1fca56 |  |
| node-price-board-flat | decoy-control | pass | tracked before d1fca56 |  |
| node-report-heap | non-leak | pass | new in d1fca56 |  |
| node-retry-scheduler | archetype-free | pass | tracked before d1fca56 |  |
| node-signal-compose-22 | decoy-control | pass | new in d1fca56 |  |
| node-signal-compose-24 | archetype-free | fail | new in d1fca56 | leak L1 (src/handler.js:12) is not confirmed |
| node-socket-heartbeat | archetype-leak | pass | tracked before d1fca56 |  |
| node-socket-heartbeat-cleared | decoy-control | pass | tracked before d1fca56 |  |
| node-template-cache | non-leak | pass | new in d1fca56 |  |
| node-token-cache | archetype-free | pass | new in d1fca56 |  |
| python-frozen-gc | archetype-free | fail | new in d1fca56 | leak L1 (journeys/worker.py:25) is not confirmed |
| python-graph-churn | non-leak | pass | tracked before d1fca56 |  |
| python-ingest-two-leaks | multi-leak | pass | tracked before d1fca56 |  |
| python-pool-fragmentation | non-leak | pass | new in d1fca56 |  |
| python-report-memo | archetype-leak | pass | tracked before d1fca56 |  |
| python-report-memo-bounded | decoy-control | pass | tracked before d1fca56 |  |
| python-retry-chain | archetype-free | pass | new in d1fca56 |  |
| python-vendor-wheel | impossible | pass | new in d1fca56 |  |
| python-zstd-options-3144 | archetype-free | fail | new in d1fca56 | leak L1 (archive/reader.py:17) is not confirmed |
| python-zstd-options-3145 | decoy-control | pass | new in d1fca56 |  |
| rust-alloc-threads | non-leak | pass | new in d1fca56 |  |
| rust-batch-buffers | non-leak | pass | tracked before d1fca56 |  |
| rust-delayed-queue | archetype-free | pass | new in d1fca56 |  |
| rust-ledger-sync | multi-leak | pass | new in d1fca56 |  |
| rust-meter-callbacks | archetype-leak | pass | tracked before d1fca56 |  |
| rust-meter-callbacks-reclaimed | decoy-control | pass | tracked before d1fca56 |  |
| rust-private-registry | impossible | pass | new in d1fca56 |  |
| rust-slot-arena | archetype-free | fail | new in d1fca56 | leak L1 (src/arena.rs:46) is not confirmed |
| rust-tree-parent-links | archetype-leak | pass | tracked before d1fca56 |  |
| rust-tree-parent-links-weak | decoy-control | pass | tracked before d1fca56 |  |
| symptom-no-target | impossible | pass | tracked before d1fca56 |  |
