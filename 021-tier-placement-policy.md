# SPEC‑021 — Tier Placement Policy (HBM / DRAM / NVMe / Object Store)

Lite LLM partitions its parameters into tiers based on access frequency and size.  Each tier may reside in high‑bandwidth GPU memory (HBM), system DRAM, local NVMe solid‑state drives, or a remote object store.  The tier placement policy governs how experts, groups and shared parameters are mapped onto these storage layers.  Effective placement is critical for performance and scalability.

## 1 Motivation

Mixture‑of‑experts models contain trillions of parameters but activate only a small subset per token【815052634867799†L135-L146】.  Keeping every expert in GPU memory would exceed available VRAM.  Hierarchical storage systems such as StrataServe cache only the working set in GPU memory, stage less frequently accessed parameters in host memory and stream cold parameters from NVMe and SSDs【75891756086750†L80-L95】.  A formal placement policy ensures predictable latency and efficient utilisation of memory tiers.

## 2 Tier Characteristics

| Tier       | Access time | Capacity | Description |
|-----------|-------------|---------|-------------|
| **Hot (HBM)**  | O(ns)      | O(GB)   | GPU device memory; highest bandwidth, lowest latency; limited capacity. |
| **Warm (DRAM)** | O(100 ns) | O(100 GB) | Host memory pinned for DMA; moderate bandwidth, more capacity. |
| **Cold (NVMe)** | O(μs)     | O(TB)   | Local NVMe SSD; high capacity; high latency; requires asynchronous I/O. |
| **Archive (Object)** | O(ms+)   | O(PB)   | Remote object storage; infinite capacity; network latency. |

## 3 Placement Rules

1. **Shared parameters:** Embeddings, attention weights and other dense parameters that are accessed every token reside in the hot tier.
2. **High‑priority experts:** Frequently activated experts (as determined by recent routing statistics) are promoted to the hot tier.
3. **Warm staging:** Less frequently used experts are kept in DRAM and prefetched into HBM when the router predicts their activation.
4. **Cold storage:** Rarely used experts (cold tiers) are stored on NVMe.  They are prefetched into DRAM or HBM on demand.
5. **Archive tiers:** Extremely rare parameters or historical snapshots reside in object storage.  They may be streamed for evaluation or fine‑tuning.

The placement policy is dynamic.  The runtime monitors expert activation frequencies and adjusts tier assignments accordingly.  StrataServe’s four‑stage pipeline overlaps network, SSD I/O and compute to hide latency【75891756086750†L80-L95】.

## 4 Placement Metadata

Each tier entry in the checkpoint manifest (SPEC‑029) includes:

* **Placement hint:** desired tier (hot, warm, cold, archive).
* **Size:** number of parameters or bytes.
* **Priority score:** used by the dynamic tier manager to decide promotions and evictions.
* **Checksum:** ensures integrity during transfers.

## 5 Policy Enforcement

The runtime implements a tier manager with the following responsibilities:

* **Promotion:** when a cold expert is activated repeatedly, promote it to warm or hot tier if capacity permits.
* **Demotion:** demote unused hot experts to warm or cold tiers to free GPU memory.
* **Prefetch:** based on routing predictions, prefetch warm or cold experts ahead of use (SPEC‑045).
* **Eviction:** when capacity constraints are reached, evict experts based on least‑recently‑used or frequency metrics (SPEC‑027).

## 6 Security Considerations

Placement decisions interact with encryption at rest (SPEC‑053) and zeroization (SPEC‑054).  Keys for decrypting cold tiers must be available before promotion.  Evicted experts must be securely wiped from GPU memory.

By defining a tier placement policy, Lite LLM balances memory capacity against latency, enabling models with up to quadrillion parameters to operate on clusters with limited GPU memory.
