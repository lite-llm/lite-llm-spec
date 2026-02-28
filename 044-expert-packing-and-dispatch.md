# SPEC‑044 — Expert Packing & Dispatch

This specification defines the procedures for **packing tokens by expert** and **dispatching** them across expert‑parallel (EP) ranks during inference and training.  The goal is to minimise communication overhead, preserve deterministic ordering, and handle the inherent sparsity and imbalance of mixture‑of‑experts workloads【529477976415087†L50-L63】.

## 1 Token Assignment and Packing

1. **Routing decisions:** After computing the HSER router (SPEC‑005), each token is assigned to a small set of experts (top‑k per token).  The output is a list of `(token_index, tier_id, group_id, expert_id, weight)` assignments.
2. **Local packing:** Each EP rank builds an array of tokens destined for each local expert.  Tokens are re‑ordered by target rank to minimise scatter/gather overhead.  The original token ordering and positions are recorded to allow deterministic unpacking.
3. **Sparse representation:** For efficiency, the packed buffer consists of contiguous memory segments for each target rank, avoiding per‑token allocations.  A header records the segment sizes and metadata.
4. **Deterministic ordering:** Sorting within each segment uses a deterministic comparator (token index then seeded hash) to ensure reproducibility across nodes【529477976415087†L50-L63】.

## 2 All‑to‑All Dispatch

1. **Initiate collective:** Once local packing is complete, all EP ranks participate in an **all‑to‑all** operation to exchange token buffers.  Each rank sends its packed segments to the corresponding destination rank.
2. **Imbalance handling:** Mixture‑of‑experts workloads exhibit extreme skew—some experts receive many tokens while others receive none【529477976415087†L50-L63】.  The dispatch protocol therefore supports variable‐sized transfers and overlapping sends/receives to keep fast ranks busy while waiting for stragglers.
3. **Deterministic metadata:** Alongside token vectors, ranks exchange metadata such as token indices, expert IDs and weights.  This metadata is compressed using run‑length encoding when possible and always sorted deterministically.
4. **Fault tolerance:** If the collective fails (network partition, process failure), the dispatch may be retried with exponential backoff.  Fatal errors trigger recovery via SPEC‑020.

## 3 Expert Execution and Return

1. **Local execution:** Each EP rank receives tokens destined for its local experts.  It unpacks the segments, retrieves the expert parameters from the hot or warm cache (SPEC‑022, SPEC‑023) and performs the expert feed‑forward function.
2. **Partial results:** The rank produces partial outputs for each token and multiplies by the router weights (softmax probabilities).  If multiple experts were selected for a token, the rank accumulates the partial contributions.
3. **Return path:** A second all‑to‑all is invoked to send the partial results back to the original token owner.  As with the dispatch, the transfer uses packed buffers with deterministic ordering.
4. **Unpacking:** The origin rank merges partial results for each token, sums them and adds to the residual stream.

## 4 Performance Considerations

* **Bandwidth saturation:** The system uses non‑blocking sends and receives to saturate network bandwidth.  Multi‑path routing (e.g., RailS) can reduce completion time for imbalanced traffic【529477976415087†L50-L63】.
* **Pipelining:** Packing, transfer and expert computation are overlapped: while rank A computes local experts, rank B may be receiving new tokens.
* **Compression:** Optional quantisation or sparsity compression may reduce payload sizes at the cost of additional compute.
* **Monitoring:** The dispatch engine emits metrics to the telemetry module (SPEC‑049) including bytes sent/received, average queue sizes and imbalance ratios.

## 5 Determinism

To maintain reproducibility:

1. **Stable sorting:** All ordering operations use a stable sort and seeded tie‑breaking (SPEC‑003).
2. **Pure functions:** Packing and unpacking must be deterministic given the same routing decisions and seed.
3. **Consistent metadata:** Both sides must agree on the number of tokens and their indices; checksums or hashes may be exchanged to detect corruption.

The expert packing and dispatch protocol ensures that mixture‑of‑experts models scale across many devices while maintaining deterministic behaviour and predictable performance even under severe load imbalance.
