# SPEC‑035 — Mixed Precision Policy

Training and inference of large neural networks benefit from mixed precision arithmetic, trading off numerical accuracy for throughput and memory savings.  Lite LLM defines a **mixed precision policy** that specifies which data types are used for parameters, activations, gradients and optimizer states, and how conversions are performed.

## 1 Rationale

Mixed precision reduces memory footprint and bandwidth requirements, allowing larger models to fit into the same hardware.  For example, using bf16 or fp16 instead of fp32 cuts memory usage in half.  However, reduced precision can lead to numerical instability if applied indiscriminately.  A well‑defined policy ensures stability while capturing most of the benefits.

## 2 Data Type Choices

* **Activations:** use bf16 (Brain Float 16) or fp16 (IEEE Float 16) for forward and backward activations.  bf16 preserves exponent range while reducing mantissa bits.
* **Weights:** store weights in bf16 or quantised int8/int4 for inference; maintain a master copy in fp32 for training when using weight updates with small step sizes.
* **Gradients:** accumulate gradients in fp32 to prevent underflow; down‑cast to bf16 before communication if needed.
* **Optimizer states:** maintain moment estimates in fp32 or bf16 depending on memory constraints.  States associated with cold experts may use lower precision or be compressed.

## 3 Casting Rules

1. **Forward:** cast inputs to bf16/fp16 before matrix multiplication.  Cast results back to fp32 if subsequent operations require higher precision (e.g., LayerNorm).
2. **Backward:** accumulate gradient contributions in fp32.  After reduction, cast to bf16/fp16 for weight updates.
3. **Weight updates:** when using optimizers like AdamW, compute updates in fp32 and cast the updated weights back to bf16/fp16.
4. **Checkpointing:** store checkpoints in the original precision of the weights; do not down‑cast to reduce file size, as precision may be lost.

## 4 Dynamic Loss Scaling

To prevent gradient underflow when using fp16, the runtime implements dynamic loss scaling.  The loss is multiplied by a scaling factor before backpropagation; gradients are divided by the same factor afterwards.  The scaling factor is adjusted during training to keep gradients within the representable range.

## 5 Quantisation for Inference

For inference, further reduce memory by quantising expert weights to int8 or int4.  Calibrated quantisation schemes preserve accuracy while reducing storage.  The runtime must support loading quantised weights from the sharded format (SPEC‑028) and dequantising them on the fly.

## 6 Monitoring Precision Effects

Monitor metrics such as training loss, gradient norms and overflow/underflow events to detect numerical instabilities.  Adjust precision or scaling factors accordingly.  Developers should test mixed precision on smaller models before deploying to large tiers.

By codifying a mixed precision policy, Lite LLM delivers the memory and speed benefits of reduced‑precision arithmetic while maintaining accuracy and training stability.
