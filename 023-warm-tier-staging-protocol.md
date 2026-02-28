# SPEC‑023 — Warm Tier Staging Protocol

Between the hot GPU cache and cold NVMe storage lies the **warm tier**, comprising host DRAM buffers pinned for direct memory access (DMA).  The warm tier serves as a staging area: it holds experts that are not hot enough to occupy HBM but are likely to be accessed soon.  This specification describes how experts are staged from cold to warm, promoted to hot and demoted back to cold.

## 1 Warm Tier Design

* **Pinned memory:** the warm tier uses page‑locked DRAM to avoid page faults during DMA transfers from GPU.  Each expert is allocated a contiguous buffer aligned to PCIe boundaries.
* **Capacity:** host memory offers tens to hundreds of gigabytes, allowing more experts to be staged than can fit on the GPU.  Staging effectively increases the hit rate of subsequent promotions to the hot tier.
* **Metadata:** each staged expert stores its placement hints, size, last access time and a priority score.

## 2 Staging Process

1. **Demand staging:** when the router selects an expert that is not present in the warm tier, the runtime reads it from cold storage (NVMe or archive).  StrataServe shows that overlapping network, SSD I/O and compute in a four‑stage pipeline hides the latency of such transfers【75891756086750†L80-L95】.
2. **Prefetch staging:** the predictive prefetcher (SPEC‑045) anticipates future expert activations and stages them ahead of time.
3. **Promotion criteria:** when an expert’s activation frequency crosses a threshold, it is promoted to the hot cache (SPEC‑022).

## 3 Demotion and Eviction

* **Demotion:** if the hot cache is full, hot experts with declining scores are demoted to the warm tier.  The demotion process moves the tensors from HBM to DRAM via DMA, freeing GPU memory.
* **Eviction:** when the warm tier is near capacity, low‑priority experts that have not been accessed recently are evicted and written back to NVMe if they are dirty.  Eviction decisions align with the global tier eviction strategy (SPEC‑027).

## 4 Consistency and Concurrency

* **Locking:** staging operations must be thread‑safe.  Use fine‑grained locks or atomic reference counts to prevent race conditions between staging and eviction.
* **Version control:** staged experts carry version identifiers; before promotion, the runtime verifies that the version has not changed on disk.
* **Data integrity:** corrupted reads trigger re‑attempts or fallback to replica shards; repeated failures cause the expert to be marked unavailable and logged.

## 5 Telemetry

Metrics captured for the warm tier include staging rate, promotion rate, eviction rate, I/O throughput and latency distributions.  These metrics assist administrators in sizing warm memory and tuning prefetch policies.

By carefully staging experts in a warm tier, Lite LLM bridges the latency gap between GPU memory and NVMe, enabling smooth promotions into the hot cache and reducing inference latency.
