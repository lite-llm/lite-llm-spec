# SPEC‑027 — Tier Eviction Strategy

Memory and storage capacities are finite.  As experts are loaded and promoted through the tier hierarchy, less frequently used experts must be evicted to make room.  The **tier eviction strategy** defines how and when to evict experts from the hot and warm tiers, ensuring efficient resource utilisation while minimising cache miss penalties.

## 1 Eviction Objectives

* **Prevent overflow:** ensure that hot and warm tiers never exceed configured capacity.
* **Maximise hit rate:** evict the least useful experts first to keep frequently activated experts resident.
* **Adapt to workload changes:** adjust to shifting access patterns over time.
* **Determinism:** avoid non‑deterministic eviction decisions that would change the order of expert loading across runs.

## 2 Eviction Metrics

For each expert, compute the following metrics:

1. **Access frequency (f):** number of activations in a sliding window.
2. **Recency (r):** time since last activation.
3. **Size (s):** bytes consumed in the cache.
4. **Tier importance (t):** a score based on the expert’s tier (e.g., more important tiers may have lower eviction penalty).

The eviction priority is a weighted combination, e.g., `priority = α/(f+1) + β·r + γ·s + δ·t`.  Lower priority experts are evicted first.

## 3 Eviction Algorithm

1. **Monitor:** continuously monitor the occupancy of hot and warm tiers.
2. **Trigger:** when occupancy exceeds capacity or insertion of a new expert would overflow, compute eviction candidates.
3. **Select:** sort experts by eviction priority and select enough candidates to free the required space.
4. **Demote or evict:** for hot tier, demote selected experts to the warm tier; for warm tier, write back to cold storage and free DRAM.
5. **Deterministic tie‑breaking:** if multiple experts have equal priority, break ties using a seeded hash of the expert identifier to ensure deterministic order.

## 4 Special Cases

* **Pinned experts:** some experts may be pinned (e.g., due to regulatory or safety requirements) and never evicted.
* **Training updates:** do not evict experts that are currently being updated; mark them as ineligible during their update window.

## 5 Integration with Prefetching

Eviction decisions feed back into the predictive prefetcher (SPEC‑045).  A high eviction rate may indicate poor prefetch accuracy or insufficient capacity.  Telemetry guides administrators to adjust thresholds or increase memory.

By formalising eviction, Lite LLM avoids thrashing and ensures that valuable experts remain in memory while rarely used experts are moved to lower tiers.
