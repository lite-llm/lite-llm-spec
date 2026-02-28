# SPEC‑046 — KV‑Cache Architecture

During inference, transformer models reuse key‑value pairs from previous tokens to avoid recomputing attention across the entire context.  Lite LLM must implement a scalable **KV‑cache** that supports long sequences, multi‑tier storage and multi‑tenant isolation.  This specification defines the structure and management of the KV‑cache.

## 1 Data Model

1. **Entries:** For each layer `l` and head `h`, the KV‑cache stores a list of keys `K_{l,h}` and values `V_{l,h}` for all processed tokens.  Each entry is associated with a token position `i` and may carry metadata such as tier ID or timestamp.
2. **Storage tiers:** The cache may be partitioned across hot (HBM), warm (DRAM) and cold (NVMe) memory depending on sequence length and resource availability.  Recent tokens and active experts reside in hot memory, while older segments may be moved to warm or cold tiers.
3. **Multi‑tenancy:** Each user/session maintains a separate KV‑cache namespace.  Entries from different tenants must not be interleaved and are isolated by a unique cache ID (SPEC‑050).

## 2 Operations

* **Append:** When a new token is processed, its key and value vectors are appended to the cache.  The operation must be O(1) in amortised cost.
* **Slice retrieval:** Attention for a new token requires retrieving all previous keys/values for the relevant layer/head.  The cache must return a contiguous slice quickly, possibly using pointer arithmetic or continuous memory mapping.
* **Eviction:** To bound memory usage, the cache evicts old entries according to policies (e.g., sliding window, memory budget).  Evicted entries may be truncated or stored in cold storage if retrieval is needed later.
* **Reset:** At the end of an inference session, the cache is cleared and memory is reclaimed.  If the same session continues (streaming), the cache persists across tokens.

## 3 Memory Layout

1. **Contiguous segments:** Keys and values are stored in contiguous arrays to maximise cache locality and enable efficient slices.  Pointers to offsets avoid copying when slicing.
2. **Tiered allocation:** When the total size exceeds the hot memory budget, the runtime migrates the least recently used segments to warm memory.  Additional tiers (e.g., NVMe) may hold very long contexts.  The hierarchical design is inspired by StrataServe’s coordinated GPU HBM, CPU DRAM and SSD serving pipeline【75891756086750†L80-L95】.
3. **Compression:** If precision reduction is enabled (SPEC‑035), keys/values may be stored in FP16 or INT8.  The cache supports mixed precision for older tokens while maintaining full precision for recent tokens.

## 4 Concurrency and Safety

* **Thread safety:** The cache supports concurrent reads/writes from multiple threads.  A read–write lock or lock‑free ring buffer ensures that appends do not block retrievals.
* **Determinism:** Cache operations must be deterministic: appends preserve token order, evictions follow a deterministic policy and do not reorder existing entries.
* **Memory safety:** Implementation in Rust uses borrow checking and explicit lifetimes to prevent use‑after‑free and data races【443264331305965†L32-L37】.

## 5 Integration with Inference Pipeline

1. **Sequence parallelism:** When using sequence parallelism, each PP rank maintains a segment of the KV‑cache.  Collectives assemble the full context when needed.
2. **Cross‑tier inference:** For very long contexts, an upstream service may stream older segments from NVMe or object store.  The KV‑cache interfaces with dynamic tier prefetching (SPEC‑045) to pre‑stage these segments.
3. **Streaming API:** The inference runtime exposes a streaming API that yields tokens while updating the KV‑cache.  Clients can feed additional tokens or end the stream to clear the cache.

By designing a tier‑aware KV‑cache with determinism, isolation and efficient eviction, Lite LLM supports long context windows and multi‑tenant serving without incurring prohibitive memory overhead.
