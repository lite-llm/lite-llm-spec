# Lite LLM Enterprise Runtime Specification 017: Cross‑Node Synchronization Guarantees

## Purpose

This specification outlines the guarantees provided by the Lite LLM runtime for synchronizing state across nodes in a distributed system.  Synchronization ensures that model parameters, optimizer states, seeds and other metadata remain consistent across all participants during training and inference.

## Synchronization Scopes

1. **Model Parameters:** Dense weights and active expert weights must be synchronized across data parallel ranks.  Tensor parallel and expert parallel partitions maintain local shards but agree on boundaries.
2. **Optimizer States:** For training, optimizer states (e.g., momentum, Adam second moments) may be sharded or replicated.  Synchronization occurs at each step to ensure consistent parameter updates.
3. **Routing Seeds:** The global routing seed and derived seeds for layers and tokens must be identical on all ranks.  Seeds are broadcast during initialization and whenever a rank joins or leaves.
4. **Tier Metadata:** The set of active tiers and their placement must be consistent.  When TierSets change, all nodes update their configuration accordingly.

## Synchronization Mechanisms

* **Barrier:** Used to ensure that all ranks reach a certain point before proceeding.  Barriers are inserted at critical points, such as between forward and backward passes.
* **Collectives:** All‑reduce, all‑gather, broadcast and reduce‑scatter operations synchronize tensors and metadata.  These operations must be deterministic (SPEC 018).
* **Checkpoint Coordination:** When saving or loading checkpoints, ranks coordinate to ensure that all shards are written or read correctly.  This may involve a global barrier and metadata exchange.
* **Dynamic Membership:** When ranks join or leave, the runtime broadcasts the updated group membership and redistributes shards.  A handshake mechanism confirms that all ranks have updated membership before continuing.

## Consistency Levels

* **Strong Consistency:** For training, parameter updates must be strongly consistent across ranks.  Updates occur only after all gradients are synchronized.
* **Eventual Consistency:** For cold tiers and archival storage, eventual consistency suffices.  If a tier is not currently active, its shards can be updated asynchronously.

## Failure Scenarios

* **Delayed Messages:** The runtime implements timeouts and resends to handle network delays.  After retries, a rank may be considered failed and removed.
* **Network Partition:** In case of partition, the runtime chooses to abort the step.  Partitioned ranks cannot make progress without violating consistency.
* **Data Corruption:** Checksums are used for every synchronized tensor.  Corruption triggers error handling (SPEC 008).

## References

Synchronization protocols draw from distributed databases and deep learning frameworks.  For further reading, see **References.md**.