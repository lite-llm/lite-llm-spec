# SPEC‑045 — Dynamic Tier Prefetching

To minimise latency when accessing cold or warm tiers, the runtime implements **dynamic prefetching**: anticipating which experts will be needed and streaming their parameters into faster memory ahead of time.  This specification defines the prefetching heuristics, scheduling, and interaction with the storage hierarchy.

## 1 Prefetch Triggers

1. **Routing predictions:** After computing initial router logits, the system predicts which tiers and experts are likely to be selected.  These predictions may be based on top‑k scores, historical usage and query semantics.
2. **TierSet policies:** The TierSet selection (SPEC‑041) defines which tiers are eligible.  Prefetching never fetches from tiers outside the current TierSet.
3. **Latency budgets:** If the latency budget is tight, prefetch may be aggressive; for looser budgets, prefetch can be more conservative to save bandwidth.
4. **Cold cache miss:** On a miss in the hot cache (SPEC‑022), prefetch may be invoked to move experts from warm or cold storage into the hot tier.

## 2 Prefetch Strategy

1. **Priority queue:** Each candidate expert is assigned a priority based on expected usage probability, size and current cache residency.  The prefetcher maintains a priority queue and issues I/O requests in decreasing order of priority.
2. **Asynchronous streaming:** Prefetching uses non‑blocking, multi‑threaded I/O.  Parameters from warm tiers (DRAM) may be moved via DMA; parameters from cold tiers (NVMe) use asynchronous read APIs.  For archive tiers (object storage), prefetch opens parallel connections to reduce latency【75891756086750†L80-L95】.
3. **Lookahead window:** The prefetcher keeps a window of *n* future tokens (e.g., 2–4 tokens ahead) and prefetches experts likely needed for those tokens.  The window size is configurable based on memory and compute trade‑offs.
4. **Cancellation:** If a prefetched expert is no longer needed (e.g., the request ended early), the transfer may be cancelled or deprioritised.  Unused prefetched data is kept in the hot cache until evicted.

## 3 Integration with Storage Hierarchy

1. **Hot tier:** Prefetched experts are loaded into HBM memory; insertion into the cache follows the policies of SPEC‑022.  Prefetched entries are marked with a special flag to prioritise them during eviction (SPEC‑027).
2. **Warm tier:** Prefetch from warm (DRAM) is cheaper; as such, the prefetcher may pre‑stage multiple candidate experts concurrently.  The prefetch queue distinguishes between warm and cold sources to schedule shorter warm transfers first.
3. **Cold tier and archive:** Prefetching from NVMe or object store may incur seconds of latency.  Therefore the system may prefetch only high‑priority experts or those repeatedly referenced in the past.  The StrataServe architecture shows how overlapping network, SSD I/O and compute can reduce overall latency【75891756086750†L80-L95】.

## 4 Feedback and Adaptation

1. **Telemetry:** The prefetcher records hit/miss rates, bytes transferred, prefetch effectiveness and wasted transfers.  These metrics feed into the inference telemetry model (SPEC‑049).
2. **Adaptive heuristics:** Based on feedback, the prefetch algorithm adjusts parameters (priority weights, lookahead window size) to balance latency and bandwidth.  For example, if a high fraction of prefetched data is unused, the lookahead window may shrink.
3. **Learning‑based models:** Optional: train a lightweight model to predict expert usage patterns from query embeddings, user history or domain tags.  This can replace or supplement heuristic scoring.

## 5 Determinism and Security

Prefetching must not violate determinism or security:

* **Deterministic caching:** The decision to prefetch must be a deterministic function of routing probabilities, seed and prefetch policy.  If randomness is used (e.g., exploration), it must be seeded.
* **Access control:** Prefetch requests respect access control (SPEC‑055) — the prefetcher must verify that the caller is authorised to access the requested tiers.  Encrypted tiers (SPEC‑053) must be decrypted after moving into a secure buffer.

Dynamic tier prefetching reduces the latency overhead of activating large, cold expert banks.  By coupling predictions with asynchronous streaming and adaptive heuristics, the runtime overlaps computation with I/O and achieves near‑constant response times even for deep TierSets【75891756086750†L80-L95】.
