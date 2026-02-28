# Lite LLM Enterprise Runtime Specification 014: Expert Parallel Specification

## Purpose

Expert parallelism (EP) distributes experts across multiple devices or nodes.  This specification defines how experts are partitioned, how tokens are dispatched to remote experts and how expert outputs are gathered.  EP is crucial for handling massive numbers of experts while keeping per‑device memory usage manageable.

## Expert Partitioning

Experts are distributed across $R_{\text{EP}}$ ranks.  Each rank stores a disjoint subset of experts from one or more tiers.  The distribution is typically uniform to balance memory usage and compute load.  For each expert identifier `(tier, group, expert)`, a hash or round‑robin assignment maps the expert to a rank.

## Token Dispatch

During routing (SPEC 005), each token is assigned to a set of experts.  Tokens may need to be dispatched to ranks holding these experts.  Dispatching involves:

1. **Packing:** On each rank, gather all tokens destined for experts on every other rank.  Represent the assignment as a list of `(token_id, expert_id)` pairs.
2. **All‑to‑All:** Use an all‑to‑all communication to send token embeddings to the destination ranks.  Each rank receives tokens for experts it owns.
3. **Execution:** On the destination rank, group tokens by expert and run the corresponding MLP.  Batch processing maximizes compute efficiency.
4. **Return:** After execution, send the expert outputs back to the originating ranks.  The originating ranks combine these outputs according to routing weights.

This dispatch pattern is repeated at each MoE layer.  Communication cost is analyzed in SPEC 015.

## Expert Memory Placement

Experts may reside in different storage tiers (SPEC 021).  EP must be aware of placement:

* **Hot Experts:** Experts stored in HBM; immediate execution.
* **Warm Experts:** Experts stored in DRAM; prefetched into HBM prior to execution.
* **Cold Experts:** Experts stored on NVMe; streamed into DRAM/HBM as needed.

The EP runtime includes a prefetcher (SPEC 045) that moves experts to HBM before they are called.  If a prefetch misses, execution stalls while the expert loads.

## Synchronization

* **Forward Pass:** Tokens are dispatched and gathered; results are combined.  No gradient synchronization occurs.
* **Backward Pass (training):** Gradients with respect to expert weights are computed on the rank owning the expert.  These gradients are then used locally or synchronized across data parallel ranks if replicated.

## Fault Tolerance

If an EP rank fails, the runtime reassigns its experts to spare devices or demotes them to cold storage.  Routing functions mask out failed ranks and recompute assignments.  Checkpointing ensures that expert weights are not lost.

## References

Expert parallelism is inspired by DeepSpeed‑MoE and Switch Transformers.  See **References.md** for more information.