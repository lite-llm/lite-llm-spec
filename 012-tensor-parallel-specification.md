# Lite LLM Enterprise Runtime Specification 012: Tensor Parallel Specification

## Purpose

This specification defines tensor parallel (TP) partitioning in the Lite LLM runtime.  TP divides the weight matrices of dense and expert layers across multiple devices to reduce memory consumption and increase compute throughput.  TP must interact seamlessly with data, pipeline and expert parallelism.

## Partitioning Strategy

Tensor parallelism splits matrices along certain dimensions and distributes the shards across TP ranks.  Two common schemes are:

* **Column Partitioning:** For a weight matrix $W\in \mathbb{R}^{m\times n}$ in a linear layer, split the columns into $r$ shards.  Each rank $i$ stores $W_i\in \mathbb{R}^{m\times n_i}$ where $\sum n_i = n$.  During forward pass, each rank computes its local output $x W_i$ and then an all‑reduce gathers partial sums.
* **Row Partitioning:** Split the rows instead.  Each rank stores $W_i\in \mathbb{R}^{m_i\times n}$.  Inputs are broadcast to all ranks, each produces a partial result, and results are concatenated.

The runtime chooses partitioning strategies based on layer type.  Attention projection matrices and expert MLP weights often use column partitioning to avoid input duplication.

## Collective Operations

Tensor parallel operations rely on collectives:

* **All‑Reduce:** After computing partial outputs, sum across TP ranks to combine results.  For example, in column partitioning the partial outputs are added to produce the full output.
* **All‑Gather:** When the next layer requires the full activation (e.g., softmax), gather outputs from all ranks.
* **Reduce‑Scatter:** In some cases, reduce and scatter outputs simultaneously to avoid intermediate buffers.

All collectives must be deterministic (SPEC 018) and exploit high‑bandwidth interconnects.

## Cross‑Parallelism Considerations

* **Data Parallelism:** TP ranks are nested inside DP groups.  Gradients are synchronized across TP ranks first, then across DP ranks.
* **Expert Parallelism:** TP operations occur inside each expert’s MLP.  Expert parameters are partitioned across devices; tokens are routed to the appropriate TP shard.
* **Pipeline Parallelism:** When layers are split across pipeline stages, each stage performs its own TP operations.  Activation checkpoints must handle partial activations correctly.

## Fault Tolerance

If a TP rank fails, the system may choose to:

* **Abort:** Terminate and restart the entire job.  TP is tightly coupled and may not gracefully degrade.
* **Data Parallel Fallback:** If replication exists, a spare device can assume the role of the failed rank.  This requires pre‑allocating extra devices and synchronizing parameters.

## References

Tensor parallelism is inspired by work in Megatron‑LM and DeepSpeed.  See **References.md** for details on implementation nuances.