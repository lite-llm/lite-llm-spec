# SPEC‑036 — Gradient Accumulation Model

Training extremely large models often requires effective batch sizes that exceed the memory capacity of a single GPU.  **Gradient accumulation** allows splitting a large batch into multiple micro‑batches, accumulating gradients across them and performing a single optimizer update.  This specification describes how Lite LLM implements gradient accumulation in a distributed environment.

## 1 Terminology

* **Micro‑batch:** a subset of the full batch processed sequentially on a device.
* **Accumulation step:** one pass over a micro‑batch where gradients are computed but parameters are not yet updated.
* **Update step:** after accumulating gradients over multiple micro‑batches, the optimizer updates the parameters.

## 2 Accumulation Procedure

1. **Zero gradients:** before processing the first micro‑batch, zero out the gradient buffers for all parameters.
2. **Forward/backward:** for each micro‑batch, perform forward and backward passes.  Accumulate gradients into the gradient buffers; do not update parameters.
3. **Synchronise:** if training is distributed across data parallel ranks, perform a deterministic all‑reduce of accumulated gradients (SPEC‑018).
4. **Update:** apply the optimizer to update parameters once all micro‑batches have been processed.
5. **Repeat:** zero gradients and repeat for the next large batch.

## 3 Scaling Learning Rates

When using gradient accumulation, the effective batch size is `micro_batch_size × accumulation_steps × num_data_parallel_ranks`.  It is common to scale the learning rate proportional to the effective batch size.  The runtime should expose configuration options to adjust learning rates automatically based on accumulation steps.

## 4 Memory Considerations

Accumulating gradients requires storing gradient buffers in full precision (fp32) until the update step.  When the model is too large, gradient sharding (SPEC‑037) may be used to store only local shards of the gradients on each rank.  Accumulated gradients for cold experts may be kept in DRAM until needed.

## 5 Determinism

The order of micro‑batches and accumulation steps must be fixed to ensure deterministic updates.  All ranks must agree on the number of accumulation steps per update.  Seeds for any random operations (e.g., dropout) should incorporate the micro‑batch index to avoid correlation across steps.

By implementing a robust gradient accumulation model, Lite LLM supports large effective batch sizes even when memory per device is limited, enabling stable training of massive models.
