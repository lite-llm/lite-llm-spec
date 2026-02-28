# Lite LLM Enterprise Runtime Specification 013: Pipeline Parallel Specification

## Purpose

This specification outlines pipeline parallelism (PP) in the Lite LLM runtime.  PP divides the model layers into sequential segments (stages) distributed across devices or nodes, enabling models that do not fit entirely on a single device while overlapping compute and communication to minimize idle time.

## Stage Partitioning

Let $L$ be the total number of transformer layers.  PP divides these layers into $S$ stages.  Each stage $s$ holds a contiguous subset of layers $[l_{s,\text{start}}, l_{s,\text{end}}]$.  Stages run on separate devices or groups of devices (tensor parallel ranks).

## Micro‑Batching

To overlap pipeline stages, the runtime splits the batch into micro‑batches (also known as pipeline bubbles).  Processing proceeds as follows:

1. Stage 1 processes micro‑batch 1 and forwards its activations to Stage 2.
2. Stage 2 processes micro‑batch 1 while Stage 1 processes micro‑batch 2, and so on.
3. Backward pass flows in the reverse direction: Stage S computes gradients for micro‑batch 1 and sends gradient activations to Stage S‑1.

The number of micro‑batches determines pipeline efficiency.  A large number of micro‑batches reduces idle time but increases memory usage.

## Communication Patterns

Between stages, the runtime communicates activation tensors and gradient tensors via point‑to‑point or collective operations.  These transfers must be deterministic and use streaming to overlap with compute.  Activations may be partitioned across tensor parallel ranks; gather/scatter operations restore full activations for the next stage.

## Activation Checkpointing

To reduce memory usage, intermediate activations can be recomputed during the backward pass rather than stored.  The runtime allows checkpointing strategies such as:

* **Uniform Checkpointing:** Checkpoint every $k$ layers; recompute activations for uncheckpointed layers during backward.
* **Custom Checkpointing:** Checkpoint based on memory budgets or dynamic heuristics.

Checkpointing interacts with pipeline parallelism: each stage may checkpoint its local layers independently.

## Latency and Throughput Considerations

Pipeline parallelism introduces pipeline bubbles—initial and final micro‑batches that do not fully utilize all stages.  To maximize throughput:

* Increase the number of micro‑batches to fill the pipeline, balancing against memory usage.
* Use asynchronous communication to overlap transfers and computation.
* Pair pipeline parallelism with tensor parallelism to keep per‑stage memory usage manageable.

## References

Pipeline parallelism is informed by the GPipe paper and is widely used in large model training.  See **References.md** for the foundational literature.