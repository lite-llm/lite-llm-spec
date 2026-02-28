# SPEC‑034 — Optimizer Abstraction Interface

The Lite LLM runtime must support multiple optimization algorithms (e.g., AdamW, Adafactor, SGD) and alternative state representations (e.g., factorised second moments).  The **optimizer abstraction interface** defines a set of traits and responsibilities that all optimizers must implement to integrate with the distributed training engine.

## 1 Design Goals

1. **Pluggability:** allow new optimizers to be added without modifying the core training loop.
2. **State sharding:** allow optimizer state (e.g., momentum buffers, variance estimates) to be sharded across devices and stored in different tiers (SPEC‑037).
3. **Precision control:** support mixed precision training by allowing state tensors to use lower precision (e.g., bf16) while maintaining high precision accumulators when needed.
4. **Determinism:** produce identical updates given the same gradients and seeds across distributed workers.

## 2 Optimizer Trait

In Rust, the optimizer abstraction can be expressed as a trait with the following essential methods:

```rust
pub trait Optimizer {
    /// Initialize optimizer state for a parameter tensor.
    fn init_state(&self, param_id: usize, shape: &[usize], dtype: DType);

    /// Apply an update to a parameter tensor given its gradient.
    fn update(
        &mut self,
        param_id: usize,
        param: &mut Tensor,
        grad: &Tensor,
        step: usize,
        learning_rate: f32,
    );

    /// Save optimizer state into a sharded file (SPEC‑037).
    fn save_state(&self, shard_writer: &mut ShardWriter);

    /// Load optimizer state from a sharded file.
    fn load_state(&mut self, shard_reader: &mut ShardReader);
}
```

## 3 Supported Optimizers

The runtime should provide default implementations of common optimizers:

* **AdamW:** decoupled weight decay with first and second moment estimates.  Memory‑heavy but widely used.
* **Adafactor:** factorised second moment to reduce memory consumption.  Suitable for very large models.
* **SGD with momentum:** simple and memory‑efficient; may be used for fine‑tuning.

Each implementation must respect the trait and manage state sharding transparently.

## 4 State Sharding and Tier Placement

Optimizer states may be large; second moment estimates can double the memory required for parameters.  The optimizer interface supports sharding states across devices and tiers (SPEC‑037).  States for rarely used experts may reside in DRAM or NVMe, while hot experts’ states remain on GPU.  The interface must provide hooks to fetch and store state shards on demand.

## 5 Determinism and Reproducibility

When using adaptive optimizers, care must be taken to ensure that sharding and reordering do not introduce nondeterminism.  State initialisation must be deterministic (e.g., fill with zeros).  The update order must be consistent across ranks.  Reductions of gradients across data parallel workers must be deterministic (SPEC‑018).  Using fixed seeds for any random perturbations (e.g., dropout) is crucial for reproducibility.

By defining a clear optimizer abstraction, Lite LLM decouples the training loop from specific optimization algorithms and allows advanced memory‑saving techniques without sacrificing determinism.
