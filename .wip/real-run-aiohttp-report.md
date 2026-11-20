# DLA report

Run: `20261120-161320-44e4`

Status: COMPLETE

Measured: 3 harness runs, 12 series, 2 captures

Coverage warning: this run exercised 0 of 68 entry points (native), so findings outside them rest on reading the code and are hypotheses, not measured leaks. Cause: no workload iteration was recorded for it. To measure: dla harness gen --lang native --driver, fill the DLA-HOOK bodies of the harness it prints so the driver calls the entry points, then dla harness run <the harness it prints>.

Coverage warning: this run exercised 0 of 49 entry points (node), so findings outside them rest on reading the code and are hypotheses, not measured leaks. Cause: no workload iteration was recorded for it. To measure: dla harness gen --lang node --driver, fill the DLA-HOOK bodies of the harness it prints so the driver calls the entry points, then dla harness run <the harness it prints>.

Coverage warning: this run exercised 286 of 785 entry points (python), so findings outside them rest on reading the code and are hypotheses, not measured leaks. Cause: most entry points were never called. To measure: fill the DLA-HOOK bodies of H3 so its driver calls the entry points, then dla harness run H3.

Time: budget 3h (auto: 2 runtimes, 916 candidates), 1h9m used.

Entry: repository

Scope: repository; excluded: none; out of scope: none

Findings: 925 enumerated, 925 dispositioned; leak hypotheses: confirmed 1, strong 2, refuted 0, suspected 917, unresolved 2; non-leak explanations: confirmed 0, refuted 3, suspected 0, unresolved 0

Residual: heap 6131.626704545454 bytes/iteration unattributed of 6131.626704545454 total (threshold 1532.9066761363636) ABOVE THRESHOLD, see O1; heap 1105.0523674242424 bytes/iteration unattributed of 1105.0523674242424 total (threshold 276.2630918560606) ABOVE THRESHOLD, see O3; heap 5774.863257575757 bytes/iteration unattributed of 5774.863257575757 total (threshold 1443.7158143939394) ABOVE THRESHOLD, see O3; heap 8.429166666666667 bytes/iteration unattributed of 8.429166666666667 total (threshold 72.69734147727273); heap 6145.262166305916 bytes/iteration unattributed of 6145.262166305916 total (threshold 1536.315541576479) ABOVE THRESHOLD, see O1; heap 6149.606250000001 bytes/iteration unattributed of 6149.606250000001 total (threshold 1537.4015625000002) ABOVE THRESHOLD, see O1; heap 9.3633658008658 bytes/iteration unattributed of 9.3633658008658 total (threshold 46.675476574675386); heap 6.849959415584415 bytes/iteration unattributed of 6.849959415584415 total (threshold 48.54238096996696); heap 6051.620454545455 bytes/iteration unattributed of 6051.620454545455 total (threshold 1512.9051136363637) ABOVE THRESHOLD, see O2; heap 6146.05340909091 bytes/iteration unattributed of 6146.05340909091 total (threshold 1536.5133522727274) ABOVE THRESHOLD, see O1

Coverage: native no iterations, 0 of 68 entry points exercised; node no iterations, 0 of 49 entry points exercised; python 3520 iterations, 286 of 785 entry points exercised

## Findings

### F102: **L-COLLECT**

- State: strong
- Origin: `census`
- Label: L-COLLECT (evidence: E21 unit-count-series, E10 census-holder)
- Growth: `probe:aiohttp.web_app._build_middlewares` 45 count/iteration (steady, gate pass, E21)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E58 intervention that removed the growth, independent of the growth record E21 (dla)
- Holder: static/global
- Owner: `aiohttp.web_app._build_middlewares`
- Region: heap
- Shape: steady
- Site: `aiohttp/web_app.py:82`
- Writers: `aiohttp/web_app.py:548`
- Annotation: lru_cache(None) on _build_middlewares keyed on (handler, apps); no cache_clear or maxsize
- Verified: no
- Merged duplicates: F8, F923, F925, F926, F928, F929, F930, F931, F932, F933, F935
- Sibling hypotheses: F102.n1
- Supporting evidence: E10, E9, E21, E19, E24, E27, E29, E53, E49, E50, E51, E54, E55, E58, E65
- Explanation (observed): The static/global lru_cache(None) on aiohttp.web_app._build_middlewares (web_app.py:82) gains one entry per distinct (handler, apps) pair at the call on web_app.py:548; nothing removes entries (no maxsize, no cache_clear), so every Application and handler seen stays reachable.
- Explanation structure: adds: aiohttp/web_app.py:548; removal: never; evidence: E21

### F117: **L-COLLECT**

- State: strong
- Origin: `census`
- Label: L-COLLECT (evidence: E21 unit-count-series, E10 census-holder)
- Growth: `probe:aiohttp.web_log.AccessLogger._FORMAT_CACHE` 5 count/iteration (steady, gate pass, E21)
- Evidence classes: context, discriminator, growth, intervention, static
- Strong: second class E59 intervention that removed the growth, independent of the growth record E21 (dla)
- Holder: static/global
- Owner: `aiohttp.web_log.AccessLogger._FORMAT_CACHE`
- Region: heap
- Shape: steady
- Site: `aiohttp/web_log.py:61`
- Writers: `aiohttp/web_log.py:75`
- Annotation: heuristic name match over the project's sources
- Verified: no
- Merged duplicates: F9, F934
- Sibling hypotheses: F117.n1
- Supporting evidence: E10, E9, E19, E21, E25, E59, E63
- Explanation (observed): The static/global dict AccessLogger._FORMAT_CACHE (web_log.py:61) gains one compiled format per distinct log_format string at web_log.py:75; nothing removes entries or bounds it, so it grows with the number of distinct formats given to AccessLogger.
- Explanation structure: adds: aiohttp/web_log.py:75; removal: never; evidence: E21

### F927: **L-COLLECT**

- State: confirmed
- Origin: `growth-series`
- Label: L-COLLECT (evidence: E47 growth-series, E46 alloc-stack)
- Growth: `stack:aiohttp.connector` 299200 bytes/dump (steady, gate pass, E47)
- Evidence classes: context, discriminator, growth
- Holder: container-field
- Owner: `aiohttp.connector._DNSCacheTable._addrs_rr`
- Region: heap
- Shape: steady
- Site: `aiohttp/connector.py:1002`
- Writers: `aiohttp/connector.py:1007`
- Removers: `aiohttp/connector.py:884`
- Annotation: _DNSCacheTable entries are added per (host,port) and removed only by an explicit clear_dns_cache call; expired entries are not evicted unless the same host is requested again
- Verified: no
- Sibling hypotheses: F927.n1
- Supporting evidence: E44, E42, E47, E45
- Explanation (observed): The container-field _DNSCacheTable._addrs_rr (connector.py:1002 allocation site) gains an entry per distinct (host, port) at connector.py:1007; entries are removed only when the application calls clear_dns_cache (connector.py:884), so expired hosts are not evicted.
- Explanation structure: adds: aiohttp/connector.py:1007; removal: conditional at aiohttp/connector.py:884; evidence: E47

Suspected, static-only findings are ordered by how likely a read-only signal makes a leak, strongest first, and keep the ledger's order within a tier: (1) a known leak pattern, (2) a census holder with writers and no removers in a registration table, container field or static/global, (3) a detector hit, (4) a census holder with writers that also has removers or sits in a thread-local, closure, native handle or blocked stack, (5) everything else, such as a holder with no writers.

### F13: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: static/global
- Owner: `aiohttp.client._RetType`
- Region: heap
- Site: `aiohttp/client.py:218`
- Writers: `aiohttp/client.py:1359`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F28: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: registration-table
- Owner: `aiohttp.client_ws.ClientWebSocketResponse._send_heartbeat`
- Region: heap
- Site: `aiohttp/client_ws.py:144`
- Writers: `aiohttp/client_ws.py:138`, `aiohttp/client_ws.py:140`, `aiohttp/web_ws.py:160`, `aiohttp/web_ws.py:162`, `aiohttp/web_ws.py:166`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F34: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: registration-table
- Owner: `aiohttp.connector.Connection._callbacks`
- Region: heap
- Site: `aiohttp/connector.py:126`
- Writers: `aiohttp/connector.py:171`, `aiohttp/connector.py:174`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F36: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.connector.TCPConnector._resolve_host_tasks`
- Region: heap
- Site: `aiohttp/connector.py:857`
- Writers: `aiohttp/connector.py:967`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F38: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: registration-table
- Owner: `aiohttp.connector.TCPConnector._resolve_host`
- Region: heap
- Site: `aiohttp/connector.py:968`
- Writers: `aiohttp/connector.py:962`, `aiohttp/connector.py:964`, `aiohttp/connector.py:978`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F39: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: registration-table
- Owner: `aiohttp.connector.TCPConnector._resolve_host`
- Region: heap
- Site: `aiohttp/connector.py:978`
- Writers: `aiohttp/connector.py:962`, `aiohttp/connector.py:964`, `aiohttp/connector.py:968`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F44: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: static/global
- Owner: `aiohttp.helpers._T`
- Region: heap
- Site: `aiohttp/helpers.py:69`
- Writers: `aiohttp/streams.py:34`, `aiohttp/web_app.py:77`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F49: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: static/global
- Owner: `aiohttp.helpers._cached_current_datetime`
- Region: heap
- Site: `aiohttp/helpers.py:524`
- Writers: `aiohttp/helpers.py:564`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F60: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: static/global
- Owner: `aiohttp.http_parser._MsgT`
- Region: heap
- Site: `aiohttp/http_parser.py:105`
- Writers: `aiohttp/http_parser.py:346`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F70: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.multipart.BodyPartReader._prev_chunk`
- Region: heap
- Site: `aiohttp/multipart.py:286`
- Writers: `aiohttp/multipart.py:346`, `aiohttp/multipart.py:386`, `aiohttp/multipart.py:414`, `aiohttp/multipart.py:419`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F72: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.multipart.MultipartWriter._parts`
- Region: heap
- Site: `aiohttp/multipart.py:815`
- Writers: `aiohttp/multipart.py:927`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F73: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: registration-table
- Owner: `aiohttp.payload.register_payload`
- Region: heap
- Site: `aiohttp/payload.py:77`
- Writers: `aiohttp/payload.py:487`, `aiohttp/payload.py:488`, `aiohttp/payload.py:489`, `aiohttp/payload.py:490`, `aiohttp/payload.py:491`, and 5 more (in the ledger annotation)
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F74: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.payload.PayloadRegistry._first`
- Region: heap
- Site: `aiohttp/payload.py:101`
- Writers: `aiohttp/payload.py:124`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F75: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.payload.PayloadRegistry._normal`
- Region: heap
- Site: `aiohttp/payload.py:102`
- Writers: `aiohttp/payload.py:126`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

### F76: **UNCLASSIFIED**

- State: suspected_static_only(not_exercised)
- Origin: `census`
- Evidence classes: static
- Holder: container-field
- Owner: `aiohttp.payload.PayloadRegistry._last`
- Region: heap
- Site: `aiohttp/payload.py:103`
- Writers: `aiohttp/payload.py:128`
- Annotation: heuristic name match over the project's sources
- Reason: not_exercised
- Remedy: no probe column tied to this holder grew in the python drivers H2/H3 (226 of 785 entry points reached); add a probe/DLA-HOOK for this holder and run dla alloc-test
- Supporting evidence: E10

#### Other static-only findings (902)

| ID | Origin | Holder | Owner or site | Reason |
|---|---|---|---|---|
| F83 | `census` | registration-table | `aiohttp.payload.PAYLOAD_REGISTRY` | not_exercised |
| F84 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F85 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F86 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F87 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F88 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F89 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F90 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F91 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F92 | `census` | registration-table | `aiohttp.payload` | not_exercised |
| F93 | `census` | static/global | `aiohttp.streams._T` | not_exercised |
| F97 | `census` | registration-table | `aiohttp.web.run_app` | not_exercised |
| F99 | `census` | static/global | `aiohttp.web_app._T` | not_exercised |
| F105 | `census` | container-field | `aiohttp.web_app.Application._subapps` | not_exercised |
| F106 | `census` | registration-table | `aiohttp.web_app.Application._add_subapp` | not_exercised |
| F107 | `census` | container-field | `aiohttp.web_app.CleanupContext._exits` | not_exercised |
| F108 | `census` | static/global | `aiohttp.web_fileresponse.CONTENT_TYPES` | not_exercised |
| F119 | `census` | static/global | `aiohttp.web_protocol.ERROR` | not_exercised |
| F125 | `census` | container-field | `aiohttp.web_request.BaseRequest._cache` | not_exercised |
| F132 | `census` | container-field | `aiohttp.web_routedef.RouteTableDef._items` | not_exercised |
| F139 | `census` | container-field | `aiohttp.web_urldispatcher.UrlDispatcher._resources` | not_exercised |
| F140 | `census` | container-field | `aiohttp.web_urldispatcher.UrlDispatcher._named_resources` | not_exercised |
| F142 | `census` | container-field | `aiohttp.web_urldispatcher.UrlDispatcher._matched_sub_app_resources` | not_exercised |
| F151 | `census` | registration-table | `aiohttp.web_ws.WebSocketResponse._send_heartbeat` | not_exercised |
| F161 | `census` | static/global | `docs.conf.extensions` | not_exercised |
| F181 | `census` | static/global | `examples.client_ws.ARGS` | not_exercised |
| F182 | `census` | static/global | `examples.curl.ARGS` | not_exercised |
| F186 | `census` | static/global | `examples.web_srv_route_deco.routes` | not_exercised |
| F187 | `census` | static/global | `examples.web_ws.sockets` | not_exercised |
| F189 | `census` | static/global | `setup.extensions` | not_exercised |
| F195 | `census` | static/global | `aiohttp._helpers.__PYX_DEFAULT_STRING_ENCODING` | not_exercised |
| F198 | `census` | container-field | `aiohttp._helpers.__pyx_obj_7aiohttp_8_helpers_reify.wrapped` | not_exercised |
| F199 | `census` | container-field | `aiohttp._helpers.__pyx_obj_7aiohttp_8_helpers_reify.name` | not_exercised |
| F201 | `census` | container-field | `aiohttp._helpers.__Pyx_CachedCFunction.type` | not_exercised |
| F202 | `census` | container-field | `aiohttp._helpers.__Pyx_CachedCFunction.method_name` | not_exercised |
| F203 | `census` | container-field | `aiohttp._helpers.__Pyx_CachedCFunction.method` | not_exercised |
| F206 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func` | not_exercised |
| F210 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_qualname` | not_exercised |
| F212 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_globals` | not_exercised |
| F213 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_code` | not_exercised |
| F214 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_closure` | not_exercised |
| F220 | `census` | container-field | `aiohttp._helpers.__Pyx_CodeObjectCacheEntry.code_object` | not_exercised |
| F221 | `census` | container-field | `aiohttp._helpers.__Pyx_CodeObjectCache.entries` | not_exercised |
| F222 | `census` | static/global | `aiohttp._helpers.__pyx_code_cache` | not_exercised |
| F297 | `census` | static/global | `aiohttp._helpers.__pyx_moduledef_slots` | not_exercised |
| F298 | `census` | static/global | `aiohttp._helpers.kwdefs` | not_exercised |
| F299 | `census` | static/global | `aiohttp._helpers.__pyx_CyFunctionType_slots` | not_exercised |
| F300 | `census` | static/global | `aiohttp._helpers.__pyx_CyFunctionType_spec` | not_exercised |
| F301 | `census` | static/global | `aiohttp._helpers.bytes` | not_exercised |
| F302 | `census` | static/global | `aiohttp._helpers.bytes` | not_exercised |
| F303 | `census` | static/global | `aiohttp._helpers.m` | not_exercised |
| F305 | `census` | static/global | `aiohttp._http_parser.__PYX_DEFAULT_STRING_ENCODING` | not_exercised |
| F310 | `census` | container-field | `aiohttp._http_parser.__pyx_opt_args_7aiohttp_12_http_parser_10HttpParser__init.timer` | not_exercised |
| F311 | `census` | container-field | `aiohttp._http_parser.__pyx_opt_args_7aiohttp_12_http_parser_10HttpParser__init.payload_exception` | not_exercised |
| F312 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.method` | not_exercised |
| F313 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.path` | not_exercised |
| F314 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.version` | not_exercised |
| F315 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.headers` | not_exercised |
| F316 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.raw_headers` | not_exercised |
| F317 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.should_close` | not_exercised |
| F318 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.compression` | not_exercised |
| F319 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.upgrade` | not_exercised |
| F320 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.chunked` | not_exercised |
| F321 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawRequestMessage.url` | not_exercised |
| F322 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.version` | not_exercised |
| F323 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.reason` | not_exercised |
| F324 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.headers` | not_exercised |
| F325 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.raw_headers` | not_exercised |
| F326 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.should_close` | not_exercised |
| F327 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.compression` | not_exercised |
| F328 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.upgrade` | not_exercised |
| F329 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_RawResponseMessage.chunked` | not_exercised |
| F330 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser.__pyx_vtab` | not_exercised |
| F331 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._cparser` | not_exercised |
| F332 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._csettings` | not_exercised |
| F333 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._raw_name` | not_exercised |
| F334 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._raw_value` | not_exercised |
| F335 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._protocol` | not_exercised |
| F336 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._loop` | not_exercised |
| F337 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._timer` | not_exercised |
| F338 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._url` | not_exercised |
| F339 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._buf` | not_exercised |
| F340 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._path` | not_exercised |
| F341 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._reason` | not_exercised |
| F342 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._headers` | not_exercised |
| F343 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._raw_headers` | not_exercised |
| F344 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._messages` | not_exercised |
| F345 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._payload` | not_exercised |
| F346 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._payload_exception` | not_exercised |
| F347 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._last_error` | not_exercised |
| F348 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser_HttpParser._content_encoding` | not_exercised |
| F349 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct__genexpr.__pyx_genexpr_arg_0` | not_exercised |
| F352 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr.__pyx_genexpr_arg_0` | not_exercised |
| F356 | `census` | container-field | `aiohttp._http_parser.__Pyx_ListComp_Append.ob_item` | not_exercised |
| F357 | `census` | container-field | `aiohttp._http_parser.__Pyx_PyList_Append.ob_item` | not_exercised |
| F360 | `census` | container-field | `aiohttp._http_parser.__Pyx_CachedCFunction.type` | not_exercised |
| F361 | `census` | container-field | `aiohttp._http_parser.__Pyx_CachedCFunction.method_name` | not_exercised |
| F362 | `census` | container-field | `aiohttp._http_parser.__Pyx_CachedCFunction.method` | not_exercised |
| F363 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func` | not_exercised |
| F367 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_qualname` | not_exercised |
| F369 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_globals` | not_exercised |
| F370 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_code` | not_exercised |
| F371 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_closure` | not_exercised |
| F377 | `census` | container-field | `aiohttp._http_parser.__Pyx_CodeObjectCacheEntry.code_object` | not_exercised |
| F378 | `census` | container-field | `aiohttp._http_parser.__Pyx_CodeObjectCache.entries` | not_exercised |
| F379 | `census` | static/global | `aiohttp._http_parser.__pyx_code_cache` | not_exercised |
| F383 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.closure` | not_exercised |
| F387 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_name` | not_exercised |
| F388 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_qualname` | not_exercised |
| F389 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_modulename` | not_exercised |
| F390 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_code` | not_exercised |
| F749 | `census` | static/global | `aiohttp._http_parser.__pyx_freelist_7aiohttp_12_http_parser_RawRequestMessage` | not_exercised |
| F752 | `census` | static/global | `aiohttp._http_parser.__pyx_freelist_7aiohttp_12_http_parser_RawResponseMessage` | not_exercised |
| F761 | `census` | static/global | `aiohttp._http_parser.__pyx_freelist_7aiohttp_12_http_parser___pyx_scope_struct__genexpr` | not_exercised |
| F764 | `census` | static/global | `aiohttp._http_parser.__pyx_freelist_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr` | not_exercised |
| F767 | `census` | static/global | `aiohttp._http_parser.__pyx_moduledef_slots` | not_exercised |
| F768 | `census` | static/global | `aiohttp._http_parser.kwdefs` | not_exercised |
| F770 | `census` | static/global | `aiohttp._http_parser.__pyx_CyFunctionType_slots` | not_exercised |
| F771 | `census` | static/global | `aiohttp._http_parser.__pyx_CyFunctionType_spec` | not_exercised |
| F772 | `census` | static/global | `aiohttp._http_parser.bytes` | not_exercised |
| F773 | `census` | static/global | `aiohttp._http_parser.bytes` | not_exercised |
| F774 | `census` | static/global | `aiohttp._http_parser.bytes` | not_exercised |
| F777 | `census` | static/global | `aiohttp._http_parser.m` | not_exercised |
| F779 | `census` | static/global | `aiohttp._http_writer.__PYX_DEFAULT_STRING_ENCODING` | not_exercised |
| F785 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func` | not_exercised |
| F789 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_qualname` | not_exercised |
| F791 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_globals` | not_exercised |
| F792 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_code` | not_exercised |
| F793 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_closure` | not_exercised |
| F799 | `census` | container-field | `aiohttp._http_writer.__Pyx_CodeObjectCacheEntry.code_object` | not_exercised |
| F800 | `census` | container-field | `aiohttp._http_writer.__Pyx_CodeObjectCache.entries` | not_exercised |
| F801 | `census` | static/global | `aiohttp._http_writer.__pyx_code_cache` | not_exercised |
| F846 | `census` | static/global | `aiohttp._http_writer.__pyx_moduledef_slots` | not_exercised |
| F847 | `census` | static/global | `aiohttp._http_writer.kwdefs` | not_exercised |
| F848 | `census` | static/global | `aiohttp._http_writer.__pyx_CyFunctionType_slots` | not_exercised |
| F849 | `census` | static/global | `aiohttp._http_writer.__pyx_CyFunctionType_spec` | not_exercised |
| F850 | `census` | static/global | `aiohttp._http_writer.bytes` | not_exercised |
| F851 | `census` | static/global | `aiohttp._http_writer.bytes` | not_exercised |
| F852 | `census` | static/global | `aiohttp._http_writer.m` | not_exercised |
| F854 | `census` | static/global | `aiohttp._websocket.__PYX_DEFAULT_STRING_ENCODING` | not_exercised |
| F860 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func` | not_exercised |
| F864 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_qualname` | not_exercised |
| F866 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_globals` | not_exercised |
| F867 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_code` | not_exercised |
| F868 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_closure` | not_exercised |
| F876 | `census` | container-field | `aiohttp._websocket.__Pyx_CodeObjectCacheEntry.code_object` | not_exercised |
| F877 | `census` | container-field | `aiohttp._websocket.__Pyx_CodeObjectCache.entries` | not_exercised |
| F878 | `census` | static/global | `aiohttp._websocket.__pyx_code_cache` | not_exercised |
| F916 | `census` | static/global | `aiohttp._websocket.__pyx_moduledef_slots` | not_exercised |
| F917 | `census` | static/global | `aiohttp._websocket.kwdefs` | not_exercised |
| F918 | `census` | static/global | `aiohttp._websocket.__pyx_CyFunctionType_slots` | not_exercised |
| F919 | `census` | static/global | `aiohttp._websocket.__pyx_CyFunctionType_spec` | not_exercised |
| F920 | `census` | static/global | `aiohttp._websocket.bytes` | not_exercised |
| F921 | `census` | static/global | `aiohttp._websocket.bytes` | not_exercised |
| F922 | `census` | static/global | `aiohttp._websocket.m` | not_exercised |
| F1 | `detector:ruff/SIM115` | — | `tests/test_client_functional.py:571` | not_exercised |
| F2 | `detector:ruff/SIM115` | — | `tests/test_client_functional.py:577` | not_exercised |
| F3 | `detector:ruff/RUF006` | — | `tests/test_client_functional.py:1076` | not_exercised |
| F4 | `detector:ruff/RUF006` | — | `tests/test_streams.py:123` | not_exercised |
| F5 | `detector:ruff/RUF006` | — | `tests/test_streams.py:806` | not_exercised |
| F6 | `detector:ruff/RUF006` | — | `tests/test_streams.py:1032` | not_exercised |
| F7 | `detector:ruff/RUF006` | — | `tests/test_streams.py:1067` | not_exercised |
| F191 | `detector:dla/native-malloc-leak-path` | — | `aiohttp/_http_parser.c:1461` | not_exercised |
| F192 | `detector:dla/native-malloc-leak-path` | — | `aiohttp/_http_writer.c:1439` | not_exercised |
| F193 | `detector:dla/native-malloc-leak-path` | — | `aiohttp/_websocket.c:1458` | not_exercised |
| F14 | `census` | static/global | `aiohttp.client.ClientSession._connector` | not_exercised |
| F16 | `census` | registration-table | `aiohttp.client_reqrep.ClientRequest._writer` | not_exercised |
| F17 | `census` | blocked-stack | `aiohttp.client_reqrep.ClientRequest.send` | not_exercised |
| F19 | `census` | static/global | `aiohttp.client_reqrep.ClientResponse._headers` | not_exercised |
| F20 | `census` | static/global | `aiohttp.client_reqrep.ClientResponse._connection` | not_exercised |
| F22 | `census` | registration-table | `aiohttp.client_reqrep.ClientResponse._writer` | not_exercised |
| F23 | `census` | registration-table | `aiohttp.client_reqrep.ClientResponse._release_connection` | not_exercised |
| F24 | `census` | blocked-stack | `aiohttp.client_ws.ClientWebSocketResponse._reset_heartbeat` | not_exercised |
| F25 | `census` | blocked-stack | `aiohttp.client_ws.ClientWebSocketResponse._send_heartbeat` | not_exercised |
| F26 | `census` | blocked-stack | `aiohttp.client_ws.ClientWebSocketResponse._send_heartbeat` | not_exercised |
| F27 | `census` | blocked-stack | `aiohttp.client_ws.ClientWebSocketResponse._send_heartbeat` | not_exercised |
| F29 | `census` | native-handle/global-ref | `aiohttp.compression_utils.ZLibCompressor.__init__` | not_exercised |
| F30 | `census` | native-handle/global-ref | `aiohttp.compression_utils.ZLibCompressor.__init__` | not_exercised |
| F35 | `census` | container-field | `aiohttp.connector.TCPConnector._throttle_dns_futures` | not_exercised |
| F37 | `census` | blocked-stack | `aiohttp.connector.TCPConnector._resolve_host` | not_exercised |
| F40 | `census` | container-field | `aiohttp.cookiejar.CookieJar._cookies` | not_exercised |
| F41 | `census` | container-field | `aiohttp.cookiejar.CookieJar._host_only_cookies` | not_exercised |
| F42 | `census` | container-field | `aiohttp.cookiejar.CookieJar._expire_heap` | not_exercised |
| F43 | `census` | container-field | `aiohttp.cookiejar.CookieJar._expirations` | not_exercised |
| F52 | `census` | registration-table | `aiohttp.helpers.TimeoutHandle._callbacks` | not_exercised |
| F54 | `census` | registration-table | `aiohttp.helpers.TimeoutHandle.timer` | not_exercised |
| F58 | `census` | static/global | `aiohttp.http_exceptions.HttpProcessingError.headers` | not_exercised |
| F61 | `census` | container-field | `aiohttp.http_parser.HttpParser._lines` | not_exercised |
| F69 | `census` | container-field | `aiohttp.multipart.BodyPartReader._unread` | not_exercised |
| F94 | `census` | registration-table | `aiohttp.streams.StreamReader._eof_callbacks` | not_exercised |
| F104 | `census` | container-field | `aiohttp.web_app.Application._state` | not_exercised |
| F121 | `census` | blocked-stack | `aiohttp.web_protocol.RequestHandler.connection_made` | not_exercised |
| F122 | `census` | blocked-stack | `aiohttp.web_protocol.RequestHandler._process_keepalive` | not_exercised |
| F123 | `census` | blocked-stack | `aiohttp.web_protocol.RequestHandler.start` | not_exercised |
| F124 | `census` | blocked-stack | `aiohttp.web_protocol.RequestHandler.start` | not_exercised |
| F133 | `census` | registration-table | `aiohttp.web_runner.BaseRunner.setup` | not_exercised |
| F134 | `census` | registration-table | `aiohttp.web_runner.BaseRunner.setup` | not_exercised |
| F135 | `census` | container-field | `aiohttp.web_server.Server._connections` | not_exercised |
| F136 | `census` | registration-table | `aiohttp.web_server.Server.connection_lost` | not_exercised |
| F137 | `census` | registration-table | `aiohttp.web_urldispatcher.Resource.add_route` | not_exercised |
| F141 | `census` | container-field | `aiohttp.web_urldispatcher.UrlDispatcher._resource_index` | not_exercised |
| F143 | `census` | registration-table | `aiohttp.web_urldispatcher.UrlDispatcher.add_resource` | not_exercised |
| F144 | `census` | registration-table | `aiohttp.web_urldispatcher.UrlDispatcher.add_resource` | not_exercised |
| F145 | `census` | registration-table | `aiohttp.web_urldispatcher.UrlDispatcher.add_static` | not_exercised |
| F147 | `census` | blocked-stack | `aiohttp.web_ws.WebSocketResponse._reset_heartbeat` | not_exercised |
| F148 | `census` | blocked-stack | `aiohttp.web_ws.WebSocketResponse._send_heartbeat` | not_exercised |
| F149 | `census` | blocked-stack | `aiohttp.web_ws.WebSocketResponse._send_heartbeat` | not_exercised |
| F150 | `census` | blocked-stack | `aiohttp.web_ws.WebSocketResponse._send_heartbeat` | not_exercised |
| F153 | `census` | blocked-stack | `aiohttp.worker.GunicornWebWorker.run` | not_exercised |
| F155 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F156 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F157 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F158 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F159 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F160 | `census` | registration-table | `aiohttp.worker.GunicornWebWorker.init_signals` | not_exercised |
| F177 | `census` | registration-table | `examples.background_tasks.listen_to_redis` | not_exercised |
| F184 | `census` | static/global | `examples.server_simple.app` | not_exercised |
| F185 | `census` | static/global | `examples.static_files.app` | not_exercised |
| F197 | `census` | static/global | `aiohttp._helpers.__pyx_m` | not_exercised |
| F200 | `census` | static/global | `aiohttp._helpers.__Pyx_RefNanny` | not_exercised |
| F208 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_dict` | not_exercised |
| F209 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_name` | not_exercised |
| F211 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_doc` | not_exercised |
| F216 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.defaults_tuple` | not_exercised |
| F217 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.defaults_kwdict` | not_exercised |
| F218 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_annotations` | not_exercised |
| F219 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_is_coroutine` | not_exercised |
| F307 | `census` | static/global | `aiohttp._http_parser.__pyx_m` | not_exercised |
| F355 | `census` | static/global | `aiohttp._http_parser.__Pyx_RefNanny` | not_exercised |
| F365 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_dict` | not_exercised |
| F366 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_name` | not_exercised |
| F368 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_doc` | not_exercised |
| F373 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.defaults_tuple` | not_exercised |
| F374 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.defaults_kwdict` | not_exercised |
| F375 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_annotations` | not_exercised |
| F376 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_is_coroutine` | not_exercised |
| F380 | `census` | container-field | `aiohttp._http_parser.__Pyx_ExcInfoStruct.exc_type` | not_exercised |
| F381 | `census` | container-field | `aiohttp._http_parser.__Pyx_ExcInfoStruct.exc_value` | not_exercised |
| F382 | `census` | container-field | `aiohttp._http_parser.__Pyx_ExcInfoStruct.exc_traceback` | not_exercised |
| F391 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_frame` | not_exercised |
| F769 | `census` | native-handle/global-ref | `aiohttp._http_parser.__Pyx_MergeVtables` | not_exercised |
| F781 | `census` | static/global | `aiohttp._http_writer.__pyx_m` | not_exercised |
| F782 | `census` | static/global | `aiohttp._http_writer.__Pyx_RefNanny` | not_exercised |
| F787 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_dict` | not_exercised |
| F788 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_name` | not_exercised |
| F790 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_doc` | not_exercised |
| F795 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.defaults_tuple` | not_exercised |
| F796 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.defaults_kwdict` | not_exercised |
| F797 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_annotations` | not_exercised |
| F798 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_is_coroutine` | not_exercised |
| F856 | `census` | static/global | `aiohttp._websocket.__pyx_m` | not_exercised |
| F859 | `census` | static/global | `aiohttp._websocket.__Pyx_RefNanny` | not_exercised |
| F862 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_dict` | not_exercised |
| F863 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_name` | not_exercised |
| F865 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_doc` | not_exercised |
| F870 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.defaults_tuple` | not_exercised |
| F871 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.defaults_kwdict` | not_exercised |
| F872 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_annotations` | not_exercised |
| F873 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_is_coroutine` | not_exercised |
| F10 | `census` | static/global | `aiohttp.client.ClientTimeout.connect` | not_exercised |
| F11 | `census` | static/global | `aiohttp.client.ClientTimeout.sock_connect` | not_exercised |
| F12 | `census` | static/global | `aiohttp.client.DEFAULT_TIMEOUT` | not_exercised |
| F15 | `census` | blocked-stack | `aiohttp.client_proto.ResponseHandler._reschedule_timeout` | not_exercised |
| F18 | `census` | static/global | `aiohttp.client_reqrep._CONNECTION_CLOSED_EXCEPTION` | not_exercised |
| F21 | `census` | container-field | `aiohttp.client_reqrep.ClientResponse._cache` | not_exercised |
| F31 | `census` | blocked-stack | `aiohttp.compression_utils.ZLibCompressor.compress` | not_exercised |
| F32 | `census` | native-handle/global-ref | `aiohttp.compression_utils.ZLibDecompressor.__init__` | not_exercised |
| F33 | `census` | blocked-stack | `aiohttp.compression_utils.ZLibDecompressor.decompress` | not_exercised |
| F45 | `census` | static/global | `aiohttp.helpers._S` | not_exercised |
| F46 | `census` | static/global | `aiohttp.helpers._SENTINEL` | not_exercised |
| F47 | `census` | static/global | `aiohttp.helpers.CHAR` | not_exercised |
| F48 | `census` | static/global | `aiohttp.helpers.parse_mimetype` | not_exercised |
| F50 | `census` | blocked-stack | `aiohttp.helpers.weakref_handle` | not_exercised |
| F51 | `census` | blocked-stack | `aiohttp.helpers.call_later` | not_exercised |
| F53 | `census` | blocked-stack | `aiohttp.helpers.TimeoutHandle.start` | not_exercised |
| F55 | `census` | static/global | `aiohttp.helpers._EXC_SENTINEL` | not_exercised |
| F56 | `census` | static/global | `aiohttp.helpers.must_be_empty_body` | not_exercised |
| F57 | `census` | static/global | `aiohttp.http.RESPONSES` | not_exercised |
| F59 | `census` | static/global | `aiohttp.http_parser.ASCIISET` | not_exercised |
| F62 | `census` | static/global | `aiohttp.http_websocket.ALLOWED_CLOSE_CODES` | not_exercised |
| F63 | `census` | static/global | `aiohttp.http_websocket.WS_CLOSED_MESSAGE` | not_exercised |
| F64 | `census` | static/global | `aiohttp.http_websocket.WS_CLOSING_MESSAGE` | not_exercised |
| F65 | `census` | static/global | `aiohttp.http_websocket._xor_table` | not_exercised |
| F66 | `census` | static/global | `aiohttp.http_writer.HttpVersion10` | not_exercised |
| F67 | `census` | static/global | `aiohttp.http_writer.HttpVersion11` | not_exercised |
| F68 | `census` | static/global | `aiohttp.multipart.Self` | not_exercised |
| F71 | `census` | container-field | `aiohttp.multipart.BodyPartReader._cache` | not_exercised |
| F77 | `census` | blocked-stack | `aiohttp.payload.IOBasePayload.write` | not_exercised |
| F78 | `census` | blocked-stack | `aiohttp.payload.IOBasePayload.write` | not_exercised |
| F79 | `census` | blocked-stack | `aiohttp.payload.IOBasePayload.write` | not_exercised |
| F80 | `census` | blocked-stack | `aiohttp.payload.TextIOPayload.write` | not_exercised |
| F81 | `census` | blocked-stack | `aiohttp.payload.TextIOPayload.write` | not_exercised |
| F82 | `census` | blocked-stack | `aiohttp.payload.TextIOPayload.write` | not_exercised |
| F95 | `census` | static/global | `aiohttp.streams.EMPTY_PAYLOAD` | not_exercised |
| F96 | `census` | static/global | `aiohttp.tracing._ParamT_contra` | not_exercised |
| F98 | `census` | blocked-stack | `aiohttp.web.run_app` | not_exercised |
| F100 | `census` | static/global | `aiohttp.web_app._U` | not_exercised |
| F101 | `census` | static/global | `aiohttp.web_app._Resource` | not_exercised |
| F103 | `census` | container-field | `aiohttp.web_app.Application._middlewares` | not_exercised |
| F109 | `census` | static/global | `aiohttp.web_fileresponse.ENCODING_EXTENSIONS` | not_exercised |
| F110 | `census` | static/global | `aiohttp.web_fileresponse.ADDITIONAL_CONTENT_TYPES` | not_exercised |
| F111 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse._sendfile_fallback` | not_exercised |
| F112 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse._sendfile_fallback` | not_exercised |
| F113 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse._sendfile_fallback` | not_exercised |
| F114 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse.prepare` | not_exercised |
| F115 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse.prepare` | not_exercised |
| F116 | `census` | blocked-stack | `aiohttp.web_fileresponse.FileResponse.prepare` | not_exercised |
| F118 | `census` | static/global | `aiohttp.web_middlewares._Func` | not_exercised |
| F120 | `census` | static/global | `aiohttp.web_protocol._PAYLOAD_ACCESS_ERROR` | not_exercised |
| F126 | `census` | blocked-stack | `aiohttp.web_request.BaseRequest.post` | not_exercised |
| F127 | `census` | blocked-stack | `aiohttp.web_request.BaseRequest.post` | not_exercised |
| F128 | `census` | blocked-stack | `aiohttp.web_request.BaseRequest.post` | not_exercised |
| F129 | `census` | blocked-stack | `aiohttp.web_request.BaseRequest.post` | not_exercised |
| F130 | `census` | static/global | `aiohttp.web_response.REASON_PHRASES` | not_exercised |
| F131 | `census` | static/global | `aiohttp.web_response.CONTENT_CODINGS` | not_exercised |
| F138 | `census` | blocked-stack | `aiohttp.web_urldispatcher.StaticResource._handle` | not_exercised |
| F146 | `census` | registration-table | `aiohttp.web_urldispatcher.UrlDispatcher.add_routes` | not_exercised |
| F152 | `census` | container-field | `aiohttp.web_ws.WebSocketResponse._reader` | not_exercised |
| F154 | `census` | blocked-stack | `aiohttp.worker.GunicornWebWorker._wait_next_notify` | not_exercised |
| F162 | `census` | static/global | `docs.conf.intersphinx_mapping` | not_exercised |
| F163 | `census` | static/global | `docs.conf.templates_path` | not_exercised |
| F164 | `census` | static/global | `docs.conf.exclude_patterns` | not_exercised |
| F165 | `census` | static/global | `docs.conf.extlinks` | not_exercised |
| F166 | `census` | static/global | `docs.conf.html_theme_options` | not_exercised |
| F167 | `census` | static/global | `docs.conf.html_css_files` | not_exercised |
| F168 | `census` | static/global | `docs.conf.html_static_path` | not_exercised |
| F169 | `census` | static/global | `docs.conf.html_sidebars` | not_exercised |
| F170 | `census` | static/global | `docs.conf.latex_elements` | not_exercised |
| F171 | `census` | static/global | `docs.conf.latex_documents` | not_exercised |
| F172 | `census` | static/global | `docs.conf.man_pages` | not_exercised |
| F173 | `census` | static/global | `docs.conf.texinfo_documents` | not_exercised |
| F174 | `census` | static/global | `docs.conf.nitpick_ignore` | not_exercised |
| F175 | `census` | registration-table | `examples.background_tasks.redis_listener` | not_exercised |
| F176 | `census` | static/global | `examples.background_tasks.websockets` | not_exercised |
| F178 | `census` | blocked-stack | `examples.background_tasks.start_background_tasks` | not_exercised |
| F179 | `census` | static/global | `examples.cli_app.args_key` | not_exercised |
| F180 | `census` | blocked-stack | `examples.client_ws.start_client` | not_exercised |
| F183 | `census` | static/global | `examples.curl.el` | not_exercised |
| F188 | `census` | static/global | `requirements.sync-direct-runtime-deps.cfg` | not_exercised |
| F194 | `census` | container-field | `aiohttp._helpers.__Pyx_StringTabEntry.p` | not_exercised |
| F196 | `census` | native-handle/global-ref | `aiohttp._helpers.__Pyx_init_sys_getdefaultencoding_params` | not_exercised |
| F204 | `census` | static/global | `aiohttp._helpers.PY_UINT64_T` | not_exercised |
| F205 | `census` | static/global | `aiohttp._helpers.PY_UINT64_T` | not_exercised |
| F207 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_weakreflist` | not_exercised |
| F215 | `census` | container-field | `aiohttp._helpers.__pyx_CyFunctionObject.func_classobj` | not_exercised |
| F223 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_d` | not_exercised |
| F224 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_b` | not_exercised |
| F225 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_cython_runtime` | not_exercised |
| F226 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_empty_tuple` | not_exercised |
| F227 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_empty_bytes` | not_exercised |
| F228 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_empty_unicode` | not_exercised |
| F229 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_CyFunctionType` | not_exercised |
| F230 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_FusedFunctionType` | not_exercised |
| F231 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_GeneratorType` | not_exercised |
| F232 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_IterableCoroutineType` | not_exercised |
| F233 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_CoroutineAwaitType` | not_exercised |
| F234 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_CoroutineType` | not_exercised |
| F235 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_type_7aiohttp_8_helpers_reify` | not_exercised |
| F236 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_ptype_7aiohttp_8_helpers_reify` | not_exercised |
| F237 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_AttributeError` | not_exercised |
| F238 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_s_Incompatible_checksums_0x_x_vs_0` | not_exercised |
| F239 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_PickleError` | not_exercised |
| F240 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s__10` | not_exercised |
| F241 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u__3` | not_exercised |
| F242 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_aiohttp__helpers` | not_exercised |
| F243 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_asyncio_coroutines` | not_exercised |
| F244 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_cache` | not_exercised |
| F245 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_cline_in_traceback` | not_exercised |
| F246 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_dict` | not_exercised |
| F247 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_dict_2` | not_exercised |
| F248 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u_disable` | not_exercised |
| F249 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_doc` | not_exercised |
| F250 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u_enable` | not_exercised |
| F251 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u_gc` | not_exercised |
| F252 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_get` | not_exercised |
| F253 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_getstate` | not_exercised |
| F254 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_import` | not_exercised |
| F255 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_is_coroutine` | not_exercised |
| F256 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u_isenabled` | not_exercised |
| F257 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_main` | not_exercised |
| F258 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_name` | not_exercised |
| F259 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_new` | not_exercised |
| F260 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_object` | not_exercised |
| F261 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pickle` | not_exercised |
| F262 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_PickleError` | not_exercised |
| F263 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_checksum` | not_exercised |
| F264 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_result` | not_exercised |
| F265 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_state` | not_exercised |
| F266 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_type` | not_exercised |
| F267 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_pyx_unpickle_reify` | not_exercised |
| F268 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reduce` | not_exercised |
| F269 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reduce_cython` | not_exercised |
| F270 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reduce_ex` | not_exercised |
| F271 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_u_reified_property_is_read_only` | not_exercised |
| F272 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reify` | not_exercised |
| F273 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reify___reduce_cython` | not_exercised |
| F274 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_reify___setstate_cython` | not_exercised |
| F275 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_self` | not_exercised |
| F276 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_setstate` | not_exercised |
| F277 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_setstate_cython` | not_exercised |
| F278 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_state` | not_exercised |
| F279 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_kp_s_stringsource` | not_exercised |
| F280 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_test` | not_exercised |
| F281 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_update` | not_exercised |
| F282 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_use_setstate` | not_exercised |
| F283 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_n_s_wrapped` | not_exercised |
| F284 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_int_88416349` | not_exercised |
| F285 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_int_124832655` | not_exercised |
| F286 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_int_250410337` | not_exercised |
| F287 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_tuple_` | not_exercised |
| F288 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_tuple__2` | not_exercised |
| F289 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_tuple__4` | not_exercised |
| F290 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_tuple__6` | not_exercised |
| F291 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_tuple__8` | not_exercised |
| F292 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_codeobj__5` | not_exercised |
| F293 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_codeobj__7` | not_exercised |
| F294 | `census` | container-field | `aiohttp._helpers.__pyx_mstate.__pyx_codeobj__9` | not_exercised |
| F295 | `census` | static/global | `aiohttp._helpers.__pyx_type_7aiohttp_8_helpers_reify_slots` | not_exercised |
| F296 | `census` | static/global | `aiohttp._helpers.__pyx_type_7aiohttp_8_helpers_reify_spec` | not_exercised |
| F304 | `census` | container-field | `aiohttp._http_parser.__Pyx_StringTabEntry.p` | not_exercised |
| F306 | `census` | native-handle/global-ref | `aiohttp._http_parser.__Pyx_init_sys_getdefaultencoding_params` | not_exercised |
| F308 | `census` | container-field | `aiohttp._http_parser.__pyx_opt_args_7cpython_11contextvars_get_value.default_value` | not_exercised |
| F309 | `census` | container-field | `aiohttp._http_parser.__pyx_opt_args_7cpython_11contextvars_get_value_no_default.default_value` | not_exercised |
| F350 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct__genexpr.__pyx_v_name` | not_exercised |
| F351 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct__genexpr.__pyx_v_val` | not_exercised |
| F353 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr.__pyx_v_name` | not_exercised |
| F354 | `census` | container-field | `aiohttp._http_parser.__pyx_obj_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr.__pyx_v_val` | not_exercised |
| F358 | `census` | static/global | `aiohttp._http_parser.PY_UINT64_T` | not_exercised |
| F359 | `census` | static/global | `aiohttp._http_parser.PY_UINT64_T` | not_exercised |
| F364 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_weakreflist` | not_exercised |
| F372 | `census` | container-field | `aiohttp._http_parser.__pyx_CyFunctionObject.func_classobj` | not_exercised |
| F384 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.gi_weakreflist` | not_exercised |
| F385 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.classobj` | not_exercised |
| F386 | `census` | container-field | `aiohttp._http_parser.__pyx_CoroutineObject.yieldfrom` | not_exercised |
| F392 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_d` | not_exercised |
| F393 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_b` | not_exercised |
| F394 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_cython_runtime` | not_exercised |
| F395 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_empty_tuple` | not_exercised |
| F396 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_empty_bytes` | not_exercised |
| F397 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_empty_unicode` | not_exercised |
| F398 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_CyFunctionType` | not_exercised |
| F399 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_FusedFunctionType` | not_exercised |
| F400 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_GeneratorType` | not_exercised |
| F401 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_IterableCoroutineType` | not_exercised |
| F402 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_CoroutineAwaitType` | not_exercised |
| F403 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_CoroutineType` | not_exercised |
| F404 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7cpython_4type_type` | not_exercised |
| F405 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7cpython_4bool_bool` | not_exercised |
| F406 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7cpython_7complex_complex` | not_exercised |
| F407 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser_RawRequestMessage` | not_exercised |
| F408 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser_RawResponseMessage` | not_exercised |
| F409 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser_HttpParser` | not_exercised |
| F410 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser_HttpRequestParser` | not_exercised |
| F411 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser_HttpResponseParser` | not_exercised |
| F412 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct__genexpr` | not_exercised |
| F413 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr` | not_exercised |
| F414 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser_RawRequestMessage` | not_exercised |
| F415 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser_RawResponseMessage` | not_exercised |
| F416 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser_HttpParser` | not_exercised |
| F417 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser_HttpRequestParser` | not_exercised |
| F418 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser_HttpResponseParser` | not_exercised |
| F419 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser___pyx_scope_struct__genexpr` | not_exercised |
| F420 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_ptype_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr` | not_exercised |
| F421 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_` | not_exercised |
| F422 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCEPT` | not_exercised |
| F423 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCEPT_CHARSET` | not_exercised |
| F424 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCEPT_ENCODING` | not_exercised |
| F425 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCEPT_LANGUAGE` | not_exercised |
| F426 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCEPT_RANGES` | not_exercised |
| F427 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_ALLOW_CREDENTIALS` | not_exercised |
| F428 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_ALLOW_HEADERS` | not_exercised |
| F429 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_ALLOW_METHODS` | not_exercised |
| F430 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_ALLOW_ORIGIN` | not_exercised |
| F431 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_EXPOSE_HEADERS` | not_exercised |
| F432 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_MAX_AGE` | not_exercised |
| F433 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_REQUEST_HEADERS` | not_exercised |
| F434 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ACCESS_CONTROL_REQUEST_METHOD` | not_exercised |
| F435 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_AGE` | not_exercised |
| F436 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ALLOW` | not_exercised |
| F437 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ALLOWED_UPGRADES` | not_exercised |
| F438 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_AUTHORIZATION` | not_exercised |
| F439 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_BadHttpMessage` | not_exercised |
| F440 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_BadStatusLine` | not_exercised |
| F441 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_BaseException` | not_exercised |
| F442 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CACHE_CONTROL` | not_exercised |
| F443 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CIMultiDict` | not_exercised |
| F444 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CIMultiDictProxy` | not_exercised |
| F445 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CIMultiDictProxy_2` | not_exercised |
| F446 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CIMultiDict_2` | not_exercised |
| F447 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONNECTION` | not_exercised |
| F448 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_DISPOSITION` | not_exercised |
| F449 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_ENCODING` | not_exercised |
| F450 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_LANGUAGE` | not_exercised |
| F451 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_LENGTH` | not_exercised |
| F452 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_LOCATION` | not_exercised |
| F453 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_MD5` | not_exercised |
| F454 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_RANGE` | not_exercised |
| F455 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_TRANSFER_ENCODING` | not_exercised |
| F456 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_CONTENT_TYPE` | not_exercised |
| F457 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_COOKIE` | not_exercised |
| F458 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ContentLengthError` | not_exercised |
| F459 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DATE` | not_exercised |
| F460 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DEBUG` | not_exercised |
| F461 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DESTINATION` | not_exercised |
| F462 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DIGEST` | not_exercised |
| F463 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DeflateBuffer` | not_exercised |
| F464 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_DeflateBuffer_2` | not_exercised |
| F465 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_EMPTY_PAYLOAD` | not_exercised |
| F466 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_EMPTY_PAYLOAD_2` | not_exercised |
| F467 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ETAG` | not_exercised |
| F468 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_EXPECT` | not_exercised |
| F469 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_EXPIRES` | not_exercised |
| F470 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_FORWARDED` | not_exercised |
| F471 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_FROM` | not_exercised |
| F472 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HOST` | not_exercised |
| F473 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_Header_name_is_too_long` | not_exercised |
| F474 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_Header_value_is_too_long` | not_exercised |
| F475 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpParser___reduce_cython` | not_exercised |
| F476 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpParser___setstate_cython` | not_exercised |
| F477 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpParser_feed_data` | not_exercised |
| F478 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpParser_feed_eof` | not_exercised |
| F479 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpParser_set_upgraded` | not_exercised |
| F480 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpRequestParser` | not_exercised |
| F481 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_HttpRequestParser` | not_exercised |
| F482 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpRequestParser___reduce_cytho` | not_exercised |
| F483 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpRequestParser___setstate_cyt` | not_exercised |
| F484 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpResponseParser` | not_exercised |
| F485 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_HttpResponseParser` | not_exercised |
| F486 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpResponseParser___reduce_cyth` | not_exercised |
| F487 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpResponseParser___setstate_cy` | not_exercised |
| F488 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion` | not_exercised |
| F489 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion10` | not_exercised |
| F490 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion10_2` | not_exercised |
| F491 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion11` | not_exercised |
| F492 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion11_2` | not_exercised |
| F493 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_HttpVersion_2` | not_exercised |
| F494 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_IF_MATCH` | not_exercised |
| F495 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_IF_MODIFIED_SINCE` | not_exercised |
| F496 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_IF_NONE_MATCH` | not_exercised |
| F497 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_IF_RANGE` | not_exercised |
| F498 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_IF_UNMODIFIED_SINCE` | not_exercised |
| F499 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_s_Incompatible_checksums_0x_x_vs_0` | not_exercised |
| F500 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_s_Incompatible_checksums_0x_x_vs_0_2` | not_exercised |
| F501 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_InvalidHeader` | not_exercised |
| F502 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_InvalidURLError` | not_exercised |
| F503 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_KEEP_ALIVE` | not_exercised |
| F504 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_LAST_EVENT_ID` | not_exercised |
| F505 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_LAST_MODIFIED` | not_exercised |
| F506 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_LINK` | not_exercised |
| F507 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_LOCATION` | not_exercised |
| F508 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_LineTooLong` | not_exercised |
| F509 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_MAX_FORWARDS` | not_exercised |
| F510 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_MemoryError` | not_exercised |
| F511 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_Not_enough_data_for_satisfy_cont` | not_exercised |
| F512 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_Not_enough_data_for_satisfy_tran` | not_exercised |
| F513 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ORIGIN` | not_exercised |
| F514 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_PRAGMA` | not_exercised |
| F515 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_PROXY_AUTHENTICATE` | not_exercised |
| F516 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_PROXY_AUTHORIZATION` | not_exercised |
| F517 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_PayloadEncodingError` | not_exercised |
| F518 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_PickleError` | not_exercised |
| F519 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RANGE` | not_exercised |
| F520 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_REFERER` | not_exercised |
| F521 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RETRY_AFTER` | not_exercised |
| F522 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_RawRequestMessage` | not_exercised |
| F523 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawRequestMessage_2` | not_exercised |
| F524 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_RawRequestMessage_2` | not_exercised |
| F525 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawRequestMessage___reduce_cytho` | not_exercised |
| F526 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawRequestMessage___setstate_cyt` | not_exercised |
| F527 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawRequestMessage__replace` | not_exercised |
| F528 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_RawResponseMessage` | not_exercised |
| F529 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawResponseMessage_2` | not_exercised |
| F530 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_RawResponseMessage_2` | not_exercised |
| F531 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawResponseMessage___reduce_cyth` | not_exercised |
| F532 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_RawResponseMessage___setstate_cy` | not_exercised |
| F533 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_ACCEPT` | not_exercised |
| F534 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_EXTENSIONS` | not_exercised |
| F535 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_KEY` | not_exercised |
| F536 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_KEY1` | not_exercised |
| F537 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_PROTOCOL` | not_exercised |
| F538 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SEC_WEBSOCKET_VERSION` | not_exercised |
| F539 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SERVER` | not_exercised |
| F540 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_SET_COOKIE` | not_exercised |
| F541 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_Status_line_is_too_long` | not_exercised |
| F542 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_StreamReader` | not_exercised |
| F543 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_StreamReader_2` | not_exercised |
| F544 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_TE` | not_exercised |
| F545 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_TRAILER` | not_exercised |
| F546 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_TRANSFER_ENCODING` | not_exercised |
| F547 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_TransferEncodingError` | not_exercised |
| F548 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_TypeError` | not_exercised |
| F549 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_UPGRADE` | not_exercised |
| F550 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_URI` | not_exercised |
| F551 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_URL` | not_exercised |
| F552 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_URL_2` | not_exercised |
| F553 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_USER_AGENT` | not_exercised |
| F554 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_VARY` | not_exercised |
| F555 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_VIA` | not_exercised |
| F556 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_WANT_DIGEST` | not_exercised |
| F557 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_WARNING` | not_exercised |
| F558 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_WWW_AUTHENTICATE` | not_exercised |
| F559 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_X_FORWARDED_FOR` | not_exercised |
| F560 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_X_FORWARDED_HOST` | not_exercised |
| F561 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_X_FORWARDED_PROTO` | not_exercised |
| F562 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s__10` | not_exercised |
| F563 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__10` | not_exercised |
| F564 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__11` | not_exercised |
| F565 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__13` | not_exercised |
| F566 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__2` | not_exercised |
| F567 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__3` | not_exercised |
| F568 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s__4` | not_exercised |
| F569 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_b__4` | not_exercised |
| F570 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__4` | not_exercised |
| F571 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_b__6` | not_exercised |
| F572 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__8` | not_exercised |
| F573 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u__9` | not_exercised |
| F574 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_add` | not_exercised |
| F575 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_after` | not_exercised |
| F576 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_after_b` | not_exercised |
| F577 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_aiohttp` | not_exercised |
| F578 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_aiohttp__http_parser` | not_exercised |
| F579 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_s_aiohttp__http_parser_pyx` | not_exercised |
| F580 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_aiohttp_helpers` | not_exercised |
| F581 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_all` | not_exercised |
| F582 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_args` | not_exercised |
| F583 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_asyncio_coroutines` | not_exercised |
| F584 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_authority` | not_exercised |
| F585 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_auto_decompress` | not_exercised |
| F586 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_before` | not_exercised |
| F587 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_begin_http_chunk_receiving` | not_exercised |
| F588 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_br` | not_exercised |
| F589 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_build` | not_exercised |
| F590 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_chunked` | not_exercised |
| F591 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_chunked` | not_exercised |
| F592 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_cline_in_traceback` | not_exercised |
| F593 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_close` | not_exercised |
| F594 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_code` | not_exercised |
| F595 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_code` | not_exercised |
| F596 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_compression` | not_exercised |
| F597 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_compression` | not_exercised |
| F598 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_data` | not_exercised |
| F599 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_data_len` | not_exercised |
| F600 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_dct` | not_exercised |
| F601 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_deflate` | not_exercised |
| F602 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_desc` | not_exercised |
| F603 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_dict` | not_exercised |
| F604 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_dict_2` | not_exercised |
| F605 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_disable` | not_exercised |
| F606 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_enable` | not_exercised |
| F607 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_encoded` | not_exercised |
| F608 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_end_http_chunk_receiving` | not_exercised |
| F609 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_errno` | not_exercised |
| F610 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_error` | not_exercised |
| F611 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ex` | not_exercised |
| F612 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_feed_data` | not_exercised |
| F613 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_feed_eof` | not_exercised |
| F614 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_format` | not_exercised |
| F615 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_fragment` | not_exercised |
| F616 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_gc` | not_exercised |
| F617 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_genexpr` | not_exercised |
| F618 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_get` | not_exercised |
| F619 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_getstate` | not_exercised |
| F620 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_gzip` | not_exercised |
| F621 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_hdrs` | not_exercised |
| F622 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_headers` | not_exercised |
| F623 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_headers` | not_exercised |
| F624 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_http_exceptions` | not_exercised |
| F625 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_http_parser` | not_exercised |
| F626 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_http_writer` | not_exercised |
| F627 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_i` | not_exercised |
| F628 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_import` | not_exercised |
| F629 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_is_coroutine` | not_exercised |
| F630 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_isenabled` | not_exercised |
| F631 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_limit` | not_exercised |
| F632 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_loop` | not_exercised |
| F633 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_lower` | not_exercised |
| F634 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_main` | not_exercised |
| F635 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_max_field_size` | not_exercised |
| F636 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_max_headers` | not_exercised |
| F637 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_max_line_size` | not_exercised |
| F638 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_messages` | not_exercised |
| F639 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_method` | not_exercised |
| F640 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_method` | not_exercised |
| F641 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_multidict` | not_exercised |
| F642 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_name` | not_exercised |
| F643 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_nb` | not_exercised |
| F644 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_new` | not_exercised |
| F645 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_s_no_default___reduce___due_to_non` | not_exercised |
| F646 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_path` | not_exercised |
| F647 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_path` | not_exercised |
| F648 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_payload_exception` | not_exercised |
| F649 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pickle` | not_exercised |
| F650 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pointer` | not_exercised |
| F651 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_protocol` | not_exercised |
| F652 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_PickleError` | not_exercised |
| F653 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_checksum` | not_exercised |
| F654 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_result` | not_exercised |
| F655 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_state` | not_exercised |
| F656 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_type` | not_exercised |
| F657 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_unpickle_RawRequestMessage` | not_exercised |
| F658 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_unpickle_RawResponseMessag` | not_exercised |
| F659 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_pyx_vtable` | not_exercised |
| F660 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_query_string` | not_exercised |
| F661 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_r` | not_exercised |
| F662 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_range` | not_exercised |
| F663 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_raw_headers` | not_exercised |
| F664 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_raw_headers` | not_exercised |
| F665 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_read_until_eof` | not_exercised |
| F666 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_reason` | not_exercised |
| F667 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_reason` | not_exercised |
| F668 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_reduce` | not_exercised |
| F669 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_reduce_cython` | not_exercised |
| F670 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_reduce_ex` | not_exercised |
| F671 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_replace` | not_exercised |
| F672 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_repr___locals_genexpr` | not_exercised |
| F673 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_response_with_body` | not_exercised |
| F674 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_ret` | not_exercised |
| F675 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_rsplit` | not_exercised |
| F676 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_self` | not_exercised |
| F677 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_send` | not_exercised |
| F678 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_set_exception` | not_exercised |
| F679 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_set_upgraded` | not_exercised |
| F680 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_setstate` | not_exercised |
| F681 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_setstate_cython` | not_exercised |
| F682 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_should_close` | not_exercised |
| F683 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_should_close` | not_exercised |
| F684 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_split` | not_exercised |
| F685 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_state` | not_exercised |
| F686 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_streams` | not_exercised |
| F687 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_s_stringsource` | not_exercised |
| F688 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_test` | not_exercised |
| F689 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_throw` | not_exercised |
| F690 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_timer` | not_exercised |
| F691 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_kp_u_unknown` | not_exercised |
| F692 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_update` | not_exercised |
| F693 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_upgrade` | not_exercised |
| F694 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_upgrade` | not_exercised |
| F695 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_url` | not_exercised |
| F696 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_url` | not_exercised |
| F697 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_use_setstate` | not_exercised |
| F698 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_val` | not_exercised |
| F699 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_version` | not_exercised |
| F700 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_version` | not_exercised |
| F701 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_u_websocket` | not_exercised |
| F702 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_n_s_yarl` | not_exercised |
| F703 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_1` | not_exercised |
| F704 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_4084195` | not_exercised |
| F705 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_21004882` | not_exercised |
| F706 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_83803150` | not_exercised |
| F707 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_209127132` | not_exercised |
| F708 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_213037754` | not_exercised |
| F709 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_int_257960607` | not_exercised |
| F710 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__5` | not_exercised |
| F711 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__7` | not_exercised |
| F712 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__12` | not_exercised |
| F713 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__14` | not_exercised |
| F714 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__15` | not_exercised |
| F715 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__16` | not_exercised |
| F716 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__18` | not_exercised |
| F717 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__20` | not_exercised |
| F718 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__24` | not_exercised |
| F719 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__26` | not_exercised |
| F720 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__28` | not_exercised |
| F721 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__30` | not_exercised |
| F722 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_tuple__37` | not_exercised |
| F723 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__17` | not_exercised |
| F724 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__19` | not_exercised |
| F725 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__21` | not_exercised |
| F726 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__22` | not_exercised |
| F727 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__23` | not_exercised |
| F728 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__25` | not_exercised |
| F729 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__27` | not_exercised |
| F730 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__29` | not_exercised |
| F731 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__31` | not_exercised |
| F732 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__32` | not_exercised |
| F733 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__33` | not_exercised |
| F734 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__34` | not_exercised |
| F735 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__35` | not_exercised |
| F736 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__36` | not_exercised |
| F737 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__38` | not_exercised |
| F738 | `census` | container-field | `aiohttp._http_parser.__pyx_mstate.__pyx_codeobj__39` | not_exercised |
| F739 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_url` | not_exercised |
| F740 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_status` | not_exercised |
| F741 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_header_field` | not_exercised |
| F742 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_header_value` | not_exercised |
| F743 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_headers_complete` | not_exercised |
| F744 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_body` | not_exercised |
| F745 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_message_begin` | not_exercised |
| F746 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_message_complete` | not_exercised |
| F747 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_chunk_header` | not_exercised |
| F748 | `census` | closure/continuation | `aiohttp._http_parser.__pyx_f_7aiohttp_12_http_parser_10HttpParser__init.on_chunk_complete` | not_exercised |
| F750 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_RawRequestMessage_slots` | not_exercised |
| F751 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_RawRequestMessage_spec` | not_exercised |
| F753 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_RawResponseMessage_slots` | not_exercised |
| F754 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_RawResponseMessage_spec` | not_exercised |
| F755 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpParser_slots` | not_exercised |
| F756 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpParser_spec` | not_exercised |
| F757 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpRequestParser_slots` | not_exercised |
| F758 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpRequestParser_spec` | not_exercised |
| F759 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpResponseParser_slots` | not_exercised |
| F760 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser_HttpResponseParser_spec` | not_exercised |
| F762 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct__genexpr_slots` | not_exercised |
| F763 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct__genexpr_spec` | not_exercised |
| F765 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr_slots` | not_exercised |
| F766 | `census` | static/global | `aiohttp._http_parser.__pyx_type_7aiohttp_12_http_parser___pyx_scope_struct_1_genexpr_spec` | not_exercised |
| F775 | `census` | static/global | `aiohttp._http_parser.__pyx_GeneratorType_slots` | not_exercised |
| F776 | `census` | static/global | `aiohttp._http_parser.__pyx_GeneratorType_spec` | not_exercised |
| F778 | `census` | container-field | `aiohttp._http_writer.__Pyx_StringTabEntry.p` | not_exercised |
| F780 | `census` | native-handle/global-ref | `aiohttp._http_writer.__Pyx_init_sys_getdefaultencoding_params` | not_exercised |
| F783 | `census` | static/global | `aiohttp._http_writer.PY_UINT64_T` | not_exercised |
| F784 | `census` | static/global | `aiohttp._http_writer.PY_UINT64_T` | not_exercised |
| F786 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_weakreflist` | not_exercised |
| F794 | `census` | container-field | `aiohttp._http_writer.__pyx_CyFunctionObject.func_classobj` | not_exercised |
| F802 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_d` | not_exercised |
| F803 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_b` | not_exercised |
| F804 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_cython_runtime` | not_exercised |
| F805 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_empty_tuple` | not_exercised |
| F806 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_empty_bytes` | not_exercised |
| F807 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_empty_unicode` | not_exercised |
| F808 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_CyFunctionType` | not_exercised |
| F809 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_FusedFunctionType` | not_exercised |
| F810 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_GeneratorType` | not_exercised |
| F811 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_IterableCoroutineType` | not_exercised |
| F812 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_CoroutineAwaitType` | not_exercised |
| F813 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_CoroutineType` | not_exercised |
| F814 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_ptype_7cpython_4type_type` | not_exercised |
| F815 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_u_` | not_exercised |
| F816 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_u_Cannot_serialize_non_str_key_r` | not_exercised |
| F817 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_u_Newline_or_carriage_return_chara` | not_exercised |
| F818 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_TypeError` | not_exercised |
| F819 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_ValueError` | not_exercised |
| F820 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_u__2` | not_exercised |
| F821 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_u__4` | not_exercised |
| F822 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s__7` | not_exercised |
| F823 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_aiohttp__http_writer` | not_exercised |
| F824 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_kp_s_aiohttp__http_writer_pyx` | not_exercised |
| F825 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_asyncio_coroutines` | not_exercised |
| F826 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_cline_in_traceback` | not_exercised |
| F827 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_format` | not_exercised |
| F828 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_headers` | not_exercised |
| F829 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_import` | not_exercised |
| F830 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_is_coroutine` | not_exercised |
| F831 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_istr` | not_exercised |
| F832 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_items` | not_exercised |
| F833 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_key` | not_exercised |
| F834 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_main` | not_exercised |
| F835 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_multidict` | not_exercised |
| F836 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_name` | not_exercised |
| F837 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_ret` | not_exercised |
| F838 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_serialize_headers` | not_exercised |
| F839 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_status_line` | not_exercised |
| F840 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_test` | not_exercised |
| F841 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_val` | not_exercised |
| F842 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_n_s_writer` | not_exercised |
| F843 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_tuple__3` | not_exercised |
| F844 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_tuple__5` | not_exercised |
| F845 | `census` | container-field | `aiohttp._http_writer.__pyx_mstate.__pyx_codeobj__6` | not_exercised |
| F853 | `census` | container-field | `aiohttp._websocket.__Pyx_StringTabEntry.p` | not_exercised |
| F855 | `census` | native-handle/global-ref | `aiohttp._websocket.__Pyx_init_sys_getdefaultencoding_params` | not_exercised |
| F857 | `census` | container-field | `aiohttp._websocket.__pyx_opt_args_7cpython_11contextvars_get_value.default_value` | not_exercised |
| F858 | `census` | container-field | `aiohttp._websocket.__pyx_opt_args_7cpython_11contextvars_get_value_no_default.default_value` | not_exercised |
| F861 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_weakreflist` | not_exercised |
| F869 | `census` | container-field | `aiohttp._websocket.__pyx_CyFunctionObject.func_classobj` | not_exercised |
| F874 | `census` | static/global | `aiohttp._websocket.PY_UINT64_T` | not_exercised |
| F875 | `census` | static/global | `aiohttp._websocket.PY_UINT64_T` | not_exercised |
| F879 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_d` | not_exercised |
| F880 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_b` | not_exercised |
| F881 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_cython_runtime` | not_exercised |
| F882 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_empty_tuple` | not_exercised |
| F883 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_empty_bytes` | not_exercised |
| F884 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_empty_unicode` | not_exercised |
| F885 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_CyFunctionType` | not_exercised |
| F886 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_FusedFunctionType` | not_exercised |
| F887 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_GeneratorType` | not_exercised |
| F888 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_IterableCoroutineType` | not_exercised |
| F889 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_CoroutineAwaitType` | not_exercised |
| F890 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_CoroutineType` | not_exercised |
| F891 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_ptype_7cpython_4type_type` | not_exercised |
| F892 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_ptype_7cpython_4bool_bool` | not_exercised |
| F893 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_ptype_7cpython_7complex_complex` | not_exercised |
| F894 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_AssertionError` | not_exercised |
| F895 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s__3` | not_exercised |
| F896 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_aiohttp__websocket` | not_exercised |
| F897 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_kp_s_aiohttp__websocket_pyx` | not_exercised |
| F898 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_asyncio_coroutines` | not_exercised |
| F899 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_cline_in_traceback` | not_exercised |
| F900 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_data` | not_exercised |
| F901 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_data_len` | not_exercised |
| F902 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_i` | not_exercised |
| F903 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_in_buf` | not_exercised |
| F904 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_is_coroutine` | not_exercised |
| F905 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_main` | not_exercised |
| F906 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_mask` | not_exercised |
| F907 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_mask_buf` | not_exercised |
| F908 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_name` | not_exercised |
| F909 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_range` | not_exercised |
| F910 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_test` | not_exercised |
| F911 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_uint32_msk` | not_exercised |
| F912 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_uint64_msk` | not_exercised |
| F913 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_n_s_websocket_mask_cython` | not_exercised |
| F914 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_tuple_` | not_exercised |
| F915 | `census` | container-field | `aiohttp._websocket.__pyx_mstate.__pyx_codeobj__2` | not_exercised |

### F190: **UNCLASSIFIED**

- State: unresolved(not_exercised)
- Origin: `detector:dla/native-malloc-leak-path`
- Evidence classes: static
- Site: `aiohttp/_helpers.c:1436`
- Suggested archetype (unconfirmed): L-OWNERSHIP
- Reason: not_exercised
- Remedy: vendored llhttp C source (native); build failed in probe; build llhttp and drive it through a native harness
- Supporting evidence: E14

### F924: **UNCLASSIFIED**

- State: unresolved(inconclusive)
- Origin: `growth-series`
- Evidence classes: growth
- Region: heap
- Shape: stair
- Reason: inconclusive
- Remedy: traceback.FrameSummary count stairs in repeated-key runs (T2); likely asyncio debug-mode source tracebacks of the harness (PYTHONASYNCIODEBUG=1); rerun the driver without debug flags to confirm
- Supporting evidence: E23

## Merged duplicates

- **F8** merged into **F102** (a duplicate of the same leak; its evidence E9 is kept on F102)
- **F9** merged into **F117** (a duplicate of the same leak; its evidence E9 is kept on F117)
- **F923** merged into **F102** (a duplicate of the same leak; its evidence E21 is kept on F102)
- **F925** merged into **F102** (a duplicate of the same leak; its evidence E27 is kept on F102)
- **F926** merged into **F102** (a duplicate of the same leak; its evidence E29 is kept on F102)
- **F928** merged into **F102** (a duplicate of the same leak; its evidence E53 is kept on F102)
- **F929** merged into **F102** (a duplicate of the same leak; its evidence E49 is kept on F102)
- **F930** merged into **F102** (a duplicate of the same leak; its evidence E50 is kept on F102)
- **F931** merged into **F102** (a duplicate of the same leak; its evidence E51 is kept on F102)
- **F932** merged into **F102** (a duplicate of the same leak; its evidence E54 is kept on F102)
- **F933** merged into **F102** (a duplicate of the same leak; its evidence E55 is kept on F102)
- **F934** merged into **F117** (a duplicate of the same leak; its evidence E63 is kept on F117)
- **F935** merged into **F102** (a duplicate of the same leak; its evidence E65 is kept on F102)

## Ruled out

- **F102.n1** (UNCLASSIFIED): discriminator E24
  - Ruled out by: plateau+purge (leak) E24
  - Hypothesis: N-WARM: the middleware cache fills once for the finite set of handlers and apps and then plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E24
  - Originally supported by: E21
- **F117.n1** (UNCLASSIFIED): discriminator E25
  - Ruled out by: plateau+purge (leak) E25
  - Hypothesis: N-WARM: the format cache fills once for the finite set of log formats and then plateaus
  - Refuted if: growth continues over the extra cycles
  - Contradicting evidence: E25
  - Originally supported by: E21
- **F927.n1** (UNCLASSIFIED): discriminator E45
  - Ruled out by: plateau+purge (leak) E45
  - Hypothesis: N-SCALE: DNS cache size scales with the number of distinct hosts requested, a bounded function of the workload
  - Refuted if: growth continues over the extra cycles with a fixed set of hosts (repeated keys)
  - Contradicting evidence: E45
  - Originally supported by: E44

## Blind spots and unknowns

### Coverage gaps

- native/bcc-memleak: blocked (bcc memleak is not driven by DLA in this version; `dla attach bpftrace` counts mapped bytes per call stack (tier 3, root))
- native/gpu-nvml: blocked (GPU memory (NVML, nvidia-smi) is not read by DLA in this version: device memory growth is a blind spot)

### Runtime blind spots

None recorded.

### Entry points not exercisable

- native: `aiohttp/_find_header.c::find_header`: build_fails
- native: `aiohttp/_helpers.c::PyInit__helpers`: build_fails
- native: `aiohttp/_http_parser.c::PyInit__http_parser`: build_fails
- native: `aiohttp/_http_parser.c::__Pyx_PyInt_As_enum__llhttp_method`: build_fails
- native: `aiohttp/_http_writer.c::PyInit__http_writer`: build_fails
- native: `aiohttp/_websocket.c::PyInit__websocket`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__debug`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_body`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_complete`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_extension_name`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_extension_name_complete`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_extension_value`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_extension_value_complete`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_chunk_header`: build_fails
- native: `vendor/llhttp/src/native/api.c::llhttp__on_header_field`: build_fails
- native: and 53 more not listed (53 x build_fails); the full list is in `coverage.json`
- node: `vendor/llhttp/bench/index.ts::request`: build_fails
- node: `vendor/llhttp/bin/build_wasm.ts::isErrorWithCode`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kCheckErr]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kOnBody]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kOnHeadersComplete]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kOnHeaders]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kOnMessageBegin]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kOnMessageComplete]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.[kReset]`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.destroy`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::HTTPParser.execute`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::cstr`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::wasm_on_body`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::wasm_on_header_field`: build_fails
- node: `vendor/llhttp/examples/wasm.ts::wasm_on_header_value`: build_fails
- node: and 34 more not listed (34 x build_fails); the full list is in `coverage.json`
- python: `aiohttp/abc.py::AbstractRouter.post_init`: no_harness
- python: `aiohttp/client.py::_BaseRequestContextManager.throw`: no_harness
- python: `aiohttp/client_reqrep.py::RequestInfo.real_url_default`: no_harness
- python: `aiohttp/compression_utils.py::ZLibDecompressor.unconsumed_tail`: no_harness
- python: `aiohttp/compression_utils.py::ZLibDecompressor.unused_data`: no_harness
- python: `aiohttp/http_websocket.py::ws_ext_gen`: no_harness
- python: `aiohttp/pytest_plugin.py::pytest_addoption`: no_harness
- python: `aiohttp/pytest_plugin.py::pytest_fixture_setup`: no_harness
- python: `aiohttp/pytest_plugin.py::pytest_generate_tests`: no_harness
- python: `aiohttp/pytest_plugin.py::pytest_pycollect_makeitem`: no_harness
- python: `aiohttp/pytest_plugin.py::pytest_pyfunc_call`: no_harness
- python: `aiohttp/pytest_plugin.py::raw_test_server`: no_harness
- python: `aiohttp/pytest_plugin.py::test_server`: no_harness
- python: `aiohttp/tracing.py::Trace.send_dns_cache_hit`: no_harness
- python: `aiohttp/tracing.py::Trace.send_dns_cache_miss`: no_harness
- python: and 13 more not listed (13 x no_harness); the full list is in `coverage.json`

### Evidence limitations and unknowns

- E8: partial coverage
- E8: limitation: database built without compiling (build-mode none)
- E10: partial coverage
- E10: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E10: limitation: writers and removers are textual name matches
- E11: partial coverage
- E11: limitation: rxjs-x rules need rxjs to resolve from the project's node_modules for type information
- E11: limitation: 6 file(s) failed to parse; first: Parsing error: /tmp/claude-0/-home-user-dla/f6814273-1d53-58fc-a01a-0b3a3a3915ac/scratchpad/u19/aiohttp/vendor/llhttp/src/llhttp.ts was included by allowDefaultProject but also was found in the projec
- E13: partial coverage
- E13: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E13: limitation: writers and removers are textual name matches
- E15: partial coverage
- E15: limitation: declaration patterns only: holders created through reflection, generated code or dependencies are not listed
- E15: limitation: writers and removers are textual name matches
- E16: limitation: entry points are matched to workloads by identifier name in the workload's own source files and, for up to two hops, in the bodies of the entry points reached; this is a name match, not a call graph
- E16: limitation: workloads were discovered by reading files; none was run
- E17: limitation: harness written; nothing was run (dla harness run measures it)
- E17: limitation: pytest-memray not installed: standard-library driver used
- E18: limitation: harness written; nothing was run (dla harness run measures it)
- E18: limitation: pytest-memray not installed: standard-library driver used
- E19: partial coverage (9 of 9 examined)
- E19: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E20: limitation: gate thresholds are provisional (not calibrated)
- E21: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E21: limitation: gate thresholds are provisional (not calibrated)
- E22: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E22: limitation: gate thresholds are provisional (not calibrated)
- E23: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E23: limitation: gate thresholds are provisional (not calibrated)
- E24: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E24: limitation: pytest-memray not installed: standard-library driver used
- E24: limitation: gate and plateau thresholds are provisional (not calibrated)
- E25: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E25: limitation: pytest-memray not installed: standard-library driver used
- E25: limitation: gate and plateau thresholds are provisional (not calibrated)
- E26: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E26: limitation: gate thresholds are provisional (not calibrated)
- E27: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E27: limitation: gate thresholds are provisional (not calibrated)
- E28: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E28: limitation: gate thresholds are provisional (not calibrated)
- E29: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E29: limitation: gate thresholds are provisional (not calibrated)
- E30: limitation: harness written; nothing was run (dla harness run measures it)
- E30: limitation: pytest-memray not installed: standard-library driver used
- E31: partial coverage (9 of 9 examined)
- E31: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E31: limitation: dropped 3 other columns (cap of 64, lowest growth first)
- E31: limitation: dropped 20 per-type columns (cap of 64, lowest growth first)
- E31: limitation: dropped 43 probe columns (cap of 64, lowest growth first)
- E32: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E32: limitation: dropped 3 other columns (cap of 64, lowest growth first)
- E32: limitation: dropped 20 per-type columns (cap of 64, lowest growth first)
- E32: limitation: dropped 43 probe columns (cap of 64, lowest growth first)
- E32: limitation: gate thresholds are provisional (not calibrated)
- E33: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E33: limitation: dropped 3 other columns (cap of 64, lowest growth first)
- E33: limitation: dropped 20 per-type columns (cap of 64, lowest growth first)
- E33: limitation: dropped 43 probe columns (cap of 64, lowest growth first)
- E33: limitation: gate thresholds are provisional (not calibrated)
- E34: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E34: limitation: dropped 3 other columns (cap of 64, lowest growth first)
- E34: limitation: dropped 20 per-type columns (cap of 64, lowest growth first)
- E34: limitation: dropped 43 probe columns (cap of 64, lowest growth first)
- E34: limitation: gate thresholds are provisional (not calibrated)
- E35: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E35: limitation: dropped 3 other columns (cap of 64, lowest growth first)
- E35: limitation: dropped 20 per-type columns (cap of 64, lowest growth first)
- E35: limitation: dropped 43 probe columns (cap of 64, lowest growth first)
- E35: limitation: gate thresholds are provisional (not calibrated)
- E36: partial coverage (1 of 3 examined)
- E36: limitation: F102: growth evidence is from S1, not S5; not subtracted
- E36: limitation: F117: growth evidence is from S1, not S5; not subtracted
- E36: limitation: off-heap: no gap series for S5; run dla gap S5 first (node.external is used when present)
- E36: limitation: threshold is provisional (not calibrated)
- E37: partial coverage (1 of 3 examined)
- E37: limitation: F102: growth measured as a count; bytes not attributed
- E37: limitation: F117: growth measured as a count; bytes not attributed
- E37: limitation: off-heap: no gap series for S1; run dla gap S1 first (node.external is used when present)
- E37: limitation: threshold is provisional (not calibrated)
- E38: partial coverage (1 of 3 examined)
- E38: limitation: F102: growth evidence is from S1, not S2; not subtracted
- E38: limitation: F117: growth evidence is from S1, not S2; not subtracted
- E38: limitation: off-heap: no gap series for S2; run dla gap S2 first (node.external is used when present)
- E38: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason inconclusive --remedy "<text>" (the growth was measured and is now under its threshold)
- E38: limitation: threshold is provisional (not calibrated)
- E39: partial coverage (1 of 3 examined)
- E39: limitation: F102: growth measured as a count; bytes not attributed
- E39: limitation: F117: growth evidence is from S1, not S3; not subtracted
- E39: limitation: off-heap: no gap series for S3; run dla gap S3 first (node.external is used when present)
- E39: limitation: threshold is provisional (not calibrated)
- E40: partial coverage (1 of 3 examined)
- E40: limitation: F102: growth measured as a count; bytes not attributed
- E40: limitation: F117: growth evidence is from S1, not S4; not subtracted
- E40: limitation: off-heap: no gap series for S4; run dla gap S4 first (node.external is used when present)
- E40: limitation: threshold is provisional (not calibrated)
- E41: partial coverage (1 of 3 examined)
- E41: limitation: F102: growth evidence is from S1, not S6; not subtracted
- E41: limitation: F117: growth evidence is from S1, not S6; not subtracted
- E41: limitation: off-heap: no gap series for S6; run dla gap S6 first (node.external is used when present)
- E41: limitation: O1 (heap) is under threshold but no confirmed finding is credited in that region to cite; resolve it with dla ledger resolve O1 --as unresolved --reason inconclusive --remedy "<text>" (the growth was measured and is now under its threshold)
- E41: limitation: threshold is provisional (not calibrated)
- E42: partial coverage (6 of 9 examined)
- E42: limitation: driver p1-fresh did not finish in 300 s and was killed (log harness/H2/logs/p1-fresh.log); the rounds written so far are kept
- E42: limitation: driver p1-fresh wrote 6 of 9 rounds
- E42: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E43: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E43: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E43: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E43: limitation: the files are ordered as given
- E44: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E44: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E44: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E44: limitation: the files are ordered as given
- E44: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E45: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E45: limitation: pytest-memray not installed: standard-library driver used
- E45: limitation: gate and plateau thresholds are provisional (not calibrated)
- E46: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E46: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E46: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E46: limitation: the files are ordered as given
- E47: limitation: tracemalloc sees allocations made after it started (interpreter start-up excluded) and only through Python's allocators; memory a C extension takes with malloc is not in it
- E47: limitation: each file lists the largest stacks by live bytes and, after the first, the stacks that grew most since the first file: a stack outside the list has no figure in that file
- E47: limitation: stacks are Python frames only, as deep as DLA_TRACEMALLOC_FRAMES (default 10); function names are the module of the frame
- E47: limitation: the files are ordered as given
- E47: limitation: capture-series rule: at least 3 captures, every figure strictly increasing across every pair, last interval at least 50% of the first (provisional, not calibrated)
- E48: limitation: driver p1-fresh did not finish in 300 s and was killed (log harness/H2/logs/p1-fresh.log); the rounds written so far are kept
- E48: limitation: driver p1-fresh wrote 6 of 9 rounds
- E48: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E48: limitation: gate thresholds are provisional (not calibrated)
- E49: limitation: driver p1-fresh did not finish in 300 s and was killed (log harness/H2/logs/p1-fresh.log); the rounds written so far are kept
- E49: limitation: driver p1-fresh wrote 6 of 9 rounds
- E49: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E49: limitation: gate thresholds are provisional (not calibrated)
- E50: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E50: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E50: limitation: gate thresholds are provisional (not calibrated)
- E51: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E51: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E51: limitation: gate thresholds are provisional (not calibrated)
- E52: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E52: limitation: gate thresholds are provisional (not calibrated)
- E53: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E53: limitation: gate thresholds are provisional (not calibrated)
- E54: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E54: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E54: limitation: gate thresholds are provisional (not calibrated)
- E55: limitation: x is the capture number in capture order; t_ms is the time since the first capture where the inputs record one
- E55: limitation: each point comes from a separate capture of one process; a figure absent from a capture has no value at that point
- E55: limitation: gate thresholds are provisional (not calibrated)
- E56: partial coverage (1 of 3 examined)
- E56: limitation: F102: growth measured as a count; bytes not attributed
- E56: limitation: F117: growth evidence is from S1, not S9; not subtracted
- E56: limitation: F927: growth evidence is from S8, not S9; not subtracted
- E56: limitation: off-heap: no gap series for S9; run dla gap S9 first (node.external is used when present)
- E56: limitation: threshold is provisional (not calibrated)
- E57: partial coverage (1 of 3 examined)
- E57: limitation: F102: no passing column is tied to it; nothing attributed
- E57: limitation: F117: growth evidence is from S1, not S7; not subtracted
- E57: limitation: F927: growth evidence is from S8, not S7; not subtracted
- E57: limitation: off-heap: no gap series for S7; run dla gap S7 first (node.external is used when present)
- E57: limitation: threshold is provisional (not calibrated)
- E58: limitation: tested in a scratch copy; the project was not changed
- E58: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E58: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E58: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E58: limitation: pytest-memray not installed: standard-library driver used
- E59: limitation: tested in a scratch copy; the project was not changed
- E59: limitation: gate thresholds are provisional (not calibrated); the 0.2 and 0.5 slope fractions are placeholders
- E59: limitation: exercised is null: the harness records entry points, not lines, so whether the edited line ran is not checked
- E59: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E59: limitation: pytest-memray not installed: standard-library driver used
- E60: partial coverage (1 of 3 examined)
- E60: limitation: F102: growth evidence is from S1, not S11; not subtracted
- E60: limitation: F117: growth evidence is from S1, not S11; not subtracted
- E60: limitation: F927: growth evidence is from S8, not S11; not subtracted
- E60: limitation: off-heap: no gap series for S11; run dla gap S11 first (node.external is used when present)
- E60: limitation: threshold is provisional (not calibrated)
- E61: partial coverage (1 of 3 examined)
- E61: limitation: F102: growth evidence is from S1, not S12; not subtracted
- E61: limitation: F117: growth evidence is from S1, not S12; not subtracted
- E61: limitation: F927: growth evidence is from S8, not S12; not subtracted
- E61: limitation: off-heap: no gap series for S12; run dla gap S12 first (node.external is used when present)
- E61: limitation: threshold is provisional (not calibrated)
- E62: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E62: limitation: gate thresholds are provisional (not calibrated)
- E63: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E63: limitation: gate thresholds are provisional (not calibrated)
- E64: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E64: limitation: gate thresholds are provisional (not calibrated)
- E65: limitation: py.traced counts only allocations made after tracemalloc started (interpreter start-up excluded)
- E65: limitation: gate thresholds are provisional (not calibrated)

## Checklist

8 answered of 8 items.

| Item | Question | Status | Evidence | Note |
|---|---|---|---|---|
| `intake.target` | Is the target a repository, a local process, an artifact, or only a symptom? | answered | E1 | — |
| `intake.scope` | Does the investigation cover the whole repository, one module or one service, and what is excluded? | answered | E1 | — |
| `intake.symptom` | What growth was observed, where, and over what time? | answered | E1 | — |
| `intake.runtimes` | Which language runtimes and processes are involved? | answered | E1 | — |
| `intake.build` | Does the project build locally? | answered | E1 | — |
| `intake.workload` | What can exercise the suspected path? | answered | E1 | — |
| `intake.consent` | Which intrusive steps has the user allowed, and are raw values permitted in evidence? | answered | E2 | — |
| `intake.budget` | How much wall-clock time may the investigation use? | answered | E1 | — |

## Evidence index

| ID | Kind | Authority | Tool | Source | Raw |
|---|---|---|---|---|---|
| E1 | run-config | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | — |
| E2 | run-config | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | — |
| E3 | probe | static | dla-probe 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | probe/probe.json |
| E4 | probe | static | dla-probe 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | probe/probe.json |
| E5 | probe | static | dla-probe 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | probe/probe.json |
| E6 | static-hit | static | ruff 0.16.9 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/python-ruff.json |
| E7 | static-hit | static | pylint 4.1.3 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/python-pylint.json |
| E8 | static-hit | static | codeql 2.27.0 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/python-codeql.json |
| E9 | static-hit | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/python-dla.json |
| E10 | census-holder | static | dla-census 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | census/python.json |
| E11 | static-hit | static | eslint 10.14.0 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/node-eslint.json |
| E12 | static-hit | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/node-dla.json |
| E13 | census-holder | static | dla-census 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | census/node.json |
| E14 | static-hit | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | sweep/native-dla.json |
| E15 | census-holder | static | dla-census 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | census/native.json |
| E16 | workload | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | workload/workloads.json |
| E17 | workload | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | harness/H1/harness.json |
| E18 | workload | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | harness/H2/harness.json |
| E19 | workload | test | dla 0.9.0-preview.2 | artifact harness/H2/harness.json (f0838458271c) | harness/H2/run-S1.json |
| E20 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S1.json (1ef14abc8f70) | trend/T1.json |
| E21 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S1.json (1ef14abc8f70) | trend/T1.json |
| E22 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S2.json (507c4201d75b) | trend/T2.json |
| E23 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S2.json (507c4201d75b) | trend/T2.json |
| E24 | discriminator | test | dla 0.9.0-preview.2 | artifact series/S3.json (aceacfe4f83e) | alloc/A1.json |
| E25 | discriminator | test | dla 0.9.0-preview.2 | artifact series/S4.json (e26bc82f5642) | alloc/A2.json |
| E26 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S3.json (aceacfe4f83e) | trend/T5.json |
| E27 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S3.json (aceacfe4f83e) | trend/T5.json |
| E28 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S4.json (e26bc82f5642) | trend/T6.json |
| E29 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S4.json (e26bc82f5642) | trend/T6.json |
| E30 | workload | static | dla 0.9.0-preview.2 | repo 285f88740f3386050f3899c07ae52429d92e1c14 | harness/H3/harness.json |
| E31 | workload | test | dla 0.9.0-preview.2 | artifact harness/H3/harness.json (d54bd25bfd3f) | harness/H3/run-S5.json |
| E32 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S5.json (e35783b2b623) | trend/T7.json |
| E33 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S5.json (e35783b2b623) | trend/T7.json |
| E34 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S6.json (da35768847b5) | trend/T8.json |
| E35 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S6.json (da35768847b5) | trend/T8.json |
| E36 | residual | test | dla 0.9.0-preview.2 | artifact series/S5.json (e35783b2b623) | gap/residual-S5-E36.json |
| E37 | residual | test | dla 0.9.0-preview.2 | artifact series/S1.json (1ef14abc8f70) | gap/residual-S1-E37.json |
| E38 | residual | test | dla 0.9.0-preview.2 | artifact series/S2.json (507c4201d75b) | gap/residual-S2-E38.json |
| E39 | residual | test | dla 0.9.0-preview.2 | artifact series/S3.json (aceacfe4f83e) | gap/residual-S3-E39.json |
| E40 | residual | test | dla 0.9.0-preview.2 | artifact series/S4.json (e26bc82f5642) | gap/residual-S4-E40.json |
| E41 | residual | test | dla 0.9.0-preview.2 | artifact series/S6.json (da35768847b5) | gap/residual-S6-E41.json |
| E42 | workload | test | dla 0.9.0-preview.2 | artifact harness/H2/harness.json (5ec9e13029d1) | harness/H2/run-S7.json |
| E43 | alloc-stack | live | py-tracemalloc 0.9.0-preview.2 | artifact allocs/A1.json (954f3f1d0179) | allocs/A1.json |
| E44 | growth-series | live | py-tracemalloc 0.9.0-preview.2 | artifact series/S8.json (f65d0f2c81a8) | series/S8.json |
| E45 | discriminator | test | dla 0.9.0-preview.2 | artifact series/S9.json (ede9e2d59a11) | alloc/A3.json |
| E46 | alloc-stack | live | py-tracemalloc 0.9.0-preview.2 | artifact allocs/A2.json (b59397d20e94) | allocs/A2.json |
| E47 | growth-series | live | py-tracemalloc 0.9.0-preview.2 | artifact series/S10.json (e48890201ab3) | series/S10.json |
| E48 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S7.json (ed2701ea31d7) | trend/T10.json |
| E49 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S7.json (ed2701ea31d7) | trend/T10.json |
| E50 | growth-series | dump | dla 0.9.0-preview.2 | artifact series/S8.json (f65d0f2c81a8) | trend/T11.json |
| E51 | unit-count-series | dump | dla 0.9.0-preview.2 | artifact series/S8.json (f65d0f2c81a8) | trend/T11.json |
| E52 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S9.json (ede9e2d59a11) | trend/T12.json |
| E53 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S9.json (ede9e2d59a11) | trend/T12.json |
| E54 | growth-series | dump | dla 0.9.0-preview.2 | artifact series/S10.json (e48890201ab3) | trend/T13.json |
| E55 | unit-count-series | dump | dla 0.9.0-preview.2 | artifact series/S10.json (e48890201ab3) | trend/T13.json |
| E56 | residual | test | dla 0.9.0-preview.2 | artifact series/S9.json (ede9e2d59a11) | gap/residual-S9-E56.json |
| E57 | residual | test | dla 0.9.0-preview.2 | artifact series/S7.json (ed2701ea31d7) | gap/residual-S7-E57.json |
| E58 | intervention | test | dla 0.9.0-preview.2 | artifact series/S11.json (a6a1bc5f8128) | intervene/I1 |
| E59 | intervention | test | dla 0.9.0-preview.2 | artifact series/S12.json (bde2008b280e) | intervene/I2 |
| E60 | residual | test | dla 0.9.0-preview.2 | artifact series/S11.json (a6a1bc5f8128) | gap/residual-S11-E60.json |
| E61 | residual | test | dla 0.9.0-preview.2 | artifact series/S12.json (bde2008b280e) | gap/residual-S12-E61.json |
| E62 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S11.json (a6a1bc5f8128) | trend/T16.json |
| E63 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S11.json (a6a1bc5f8128) | trend/T16.json |
| E64 | growth-series | test | dla 0.9.0-preview.2 | artifact series/S12.json (bde2008b280e) | trend/T17.json |
| E65 | unit-count-series | test | dla 0.9.0-preview.2 | artifact series/S12.json (bde2008b280e) | trend/T17.json |
