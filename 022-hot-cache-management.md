# SPEC‑022 — Hot Cache Management

Hot cache management governs how the runtime manages the limited high‑bandwidth memory on GPUs (HBM) to maximise hit rates for frequently used experts and minimise latency for rare accesses.  Effective caching is essential to keep the working set of experts available for immediate execution while evicting stale entries to make room for new data.

## 1 Cache Goals

1. **High hit rate:** maximise the fraction of expert calls that find their parameters in HBM, avoiding expensive transfers from DRAM or NVMe.
2. **Low overhead:** maintain cache metadata and replacement algorithms with minimal computational overhead.
3. **Predictive prefetching:** prefetch experts based on routing predictions to hide latency.
4. **Security compliance:** ensure that evicted parameters are securely cleared (SPEC‑054) and that encrypted cold tiers remain protected (SPEC‑053).

## 2 Cache Structure

The hot cache is a key‑value store on HBM where keys are expert identifiers (tier, group, expert) and values are parameter tensors.  Each entry stores:

* **Size:** bytes used in HBM.
* **Access timestamp or counter:** for replacement decisions.
* **Reference count:** number of active kernels using the expert (prevents premature eviction).

## 3 Insertion Policy

* **On first activation:** if an expert is not in the cache, the runtime loads it from warm or cold tier into a staging buffer then inserts it into the cache.
* **Prefetch:** the predictive prefetcher (SPEC‑045) may insert an expert into the cache before it is needed.
* **Capacity check:** if inserting an expert would exceed cache capacity, run the eviction policy to free space.

## 4 Replacement Policy

The cache uses a hybrid eviction policy combining Least Recently Used (LRU) and frequency counting:

1. **Frequency score:** for each expert, compute `score = accesses / (1 + time_since_last_use)`.  Higher scores indicate hot experts.
2. **Eviction candidate:** choose the lowest score expert that is not currently referenced by running kernels.
3. **Demotion:** demote evicted experts to the warm or cold tier depending on their historical frequency.

The StrataServe architecture demonstrates that caching only the working parameters on GPU and staging others across DRAM and SSD enables scaling without overprovisioning compute【75891756086750†L80-L95】.

## 5 Consistency

* **Write‑through:** updates to expert parameters during training propagate immediately to lower tiers to ensure consistency across the hierarchy.
* **Versioning:** cached experts include version numbers to detect staleness.  If the version has changed on disk, reload the expert.

## 6 Metrics and Telemetry

The runtime collects cache metrics such as hit rate, eviction count, average residence time and prefetch accuracy.  These metrics feed back into the placement and prefetch policies.

Through careful hot cache management, Lite LLM ensures that most expert activations remain in high‑bandwidth memory, hiding latency from storage tiers and enabling high throughput execution.
