# SPEC‑056 — Deterministic Audit Logging

To satisfy compliance, debugging and forensic requirements, Lite LLM maintains a **deterministic audit log** that records security‑relevant and critical operational events.  This specification defines the structure, contents and security properties of the audit log.

## 1 Logged Events

1. **Model loading:** Timestamps, model IDs, manifest hashes, signatures and verification outcomes (SPEC‑052).
2. **Tier activations:** TierSet selections (SPEC‑041), including any budget constraints and cost weights (SPEC‑048).
3. **Routing decisions:** Seeds used for routing and sampling (SPEC‑003), selected experts per token (hashed or anonymised), and load balancing metrics (SPEC‑032).
4. **Access control checks:** Authentication attempts, authorisation results, tier access decisions (SPEC‑055).
5. **Prefetch and caching:** Prefetch requests, hits/misses, bytes transferred and eviction events (SPEC‑045, SPEC‑022).
6. **Errors and exceptions:** Recoverable and fatal errors (SPEC‑008), including stack traces if available.
7. **Security events:** Integrity verification failures (SPEC‑052), encryption/decryption operations (SPEC‑053), zeroisation events (SPEC‑054).

## 2 Log Properties

1. **Determinism:** The log sequence is deterministic for a given run: identical requests with the same seed produce the same log entries in the same order.  Non‑determinism (e.g., concurrency differences) is controlled via seeded tie‑breaking.
2. **Tamper evidence:** Logs are written append‑only with sequence numbers and cryptographic hashes chaining entries.  A root hash may be stored externally to detect tampering.
3. **Confidentiality:** Sensitive data (user text, keys, raw weights) is not logged.  Expert identifiers may be hashed to protect model internals.
4. **Integrity:** Logs are signed or MACed so that modifications are detectable.  Only trusted logging agents have write permissions.
5. **Retention:** Logs are retained according to configurable policies and may be pruned or archived after a defined period.

## 3 Storage and Access

* **Local buffer:** During a run, logs are stored in a memory buffer or local disk.  On completion or at intervals, logs are flushed to secure storage.
* **Central aggregator:** Logs from multiple ranks and nodes are aggregated.  Sequence numbers include node IDs to preserve ordering.
* **Access control:** Only authorised auditors or operators can read audit logs.  Requests to view logs are themselves logged.

## 4 Compliance Alignment

Deterministic audit logs support regulatory requirements for traceability, accountability and forensic analysis.  Combining secure logging with memory‑safe implementation reduces the risk of undiscovered vulnerabilities【443264331305965†L32-L37】.

By capturing a complete, tamper‑evident record of critical events, Lite LLM provides transparency and accountability, enabling operators to investigate incidents, meet compliance obligations and refine system policies.
