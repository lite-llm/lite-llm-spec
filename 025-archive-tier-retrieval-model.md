# SPEC‑025 — Archive Tier Retrieval Model

The **archive tier** represents the deepest layer of the storage hierarchy, backed by remote object stores or cloud blob services.  It is used for parameters that are rarely activated, historical snapshots and experimental branches.  This specification defines how the runtime retrieves data from the archive tier and integrates it into the working set.

## 1 Archive Tier Characteristics

* **Practically unlimited capacity:** object stores can hold petabytes of data across multiple replicas.
* **High latency:** retrieval times are measured in milliseconds to seconds, depending on network conditions.
* **Durability:** archive data is protected with erasure coding and multiple geographic replicas.
* **Cost:** reads are metered; prefetching incurs monetary cost, so it must be judicious.

## 2 Use Cases for the Archive Tier

1. **Long‑tail experts:** extremely rare experts that may only activate under unusual inputs.
2. **Versioned checkpoints:** previous model versions retained for reproducibility or rollback.
3. **Research branches:** experimental experts not part of the production tier set.
4. **Cold start:** initial loading of newly added tiers before they are used.

## 3 Retrieval Protocol

1. **Metadata lookup:** consult the checkpoint manifest (SPEC‑029) to locate the object key, size and checksum for the expert.
2. **Authentication:** use secure credentials managed by the key management system (SPEC‑057) to authenticate to the object store.
3. **Range fetch:** if supported, issue ranged GET requests to download only the needed portion (e.g., a single expert) rather than the entire blob.
4. **Streaming:** pipe the data into the cold or warm tier buffers while verifying checksums.
5. **Fallback:** if retrieval fails due to network or service errors, the runtime retries with backoff and logs the event.  Persistent failures mark the expert as unavailable.

## 4 Caching and Promotion

Experts fetched from the archive tier are staged through the cold and warm tiers before being promoted to the hot cache.  Because archive retrieval is expensive, the predictive prefetch engine must have high confidence before issuing such requests.  The runtime should avoid frequent archive accesses by promoting experts that cross activation thresholds.

## 5 Security and Compliance

Archive data is encrypted at rest (SPEC‑053).  Retrieval involves decryption keys, which are stored in the key management system.  Audit logs (SPEC‑056) record every archive fetch, providing traceability for compliance.

By providing a structured retrieval model for the archive tier, Lite LLM can safely and efficiently access deep storage without compromising performance or security.
