# Lite LLM Enterprise Runtime Specification 011: Data Parallel Specification

## Purpose

This specification defines the data parallel (DP) layer of the Lite LLM runtime.  DP replicates model parameters across multiple workers and divides input batches among them, enabling scalable throughput.  It also describes gradient synchronization and related infrastructure for training.

## Replication & Sharding

In data parallelism, each worker (or rank) maintains a replica of the model’s dense parameters and local copies of active experts.  Input batches are split into $N$ shards, each processed by a separate worker.  Each worker executes the full forward and backward pass on its shard and accumulates gradients locally.

### Local Batch Execution

1. **Forward Pass:** Compute activations for the shard, including routing and expert execution.  Active experts may communicate with other ranks via expert parallelism (SPEC 014).
2. **Backward Pass:** Compute gradients with respect to parameters and routing weights.  Gradients are accumulated in local buffers.
3. **Gradient Scaling:** Scale gradients by 1/$N$ to maintain equivalent learning rate when summing across ranks.

### Gradient Synchronization

After computing local gradients, workers participate in collective operations to produce a globally consistent update:

* **All‑Reduce:** For dense parameters, use an all‑reduce to sum gradients across all DP ranks.  After reduction, each rank updates its local copy.
* **Expert Gradients:** For experts residing on other ranks, use all‑to‑all gather to collect gradients only for experts assigned to each rank.  After summing, the gradients are applied locally.
* **Optimizer State:** Optimizer states may be sharded (SPEC 037) or replicated.  If replicated, they participate in the all‑reduce; if sharded, synchronization occurs only within the shard.

## Mixed Precision & Loss Scaling

Data parallelism interacts with mixed precision training.  To avoid underflow, loss scaling may be used on each rank prior to gradient computation.  After all‑reduce, gradients are de‑scaled before optimizer updates.  Gradient clipping is applied after reduction to ensure stability.

## Fault Tolerance & Elasticity

Data parallel groups can be elastic—ranks may be added or removed during training:

* **Join:** A new rank loads the current checkpoint, including dense weights and required tier shards.  It participates in gradient reduction on the next step.
* **Drop:** A failing rank is removed from the group.  Remaining ranks re‑normalize gradient scaling by adjusting the scaling factor (1/(N-1)).
* **Consistency:** Deterministic seeds must be updated when ranks join or leave, but routing decisions remain consistent because the seed is broadcast.

## References

DP in Lite LLM builds upon standard data parallel strategies used in distributed deep learning frameworks.  See **References.md** for foundational papers.