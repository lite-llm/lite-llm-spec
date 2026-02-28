# SPEC‑043 — Token Routing Execution Pipeline

Inference in Lite LLM involves more than executing matrix multiplications; it also includes routing each token through multiple levels of the mixture‑of‑experts hierarchy.  The **token routing execution pipeline** orchestrates these operations to minimise latency and maximise throughput.

## 1 Overview

For each input sequence and each transformer layer, the following steps occur:

1. **Attention and dense compute:** compute attention outputs and apply the dense portions of the layer (shared across all tokens).
2. **Routing scores:** compute routing scores for each token at the tier, group and expert levels (SPEC‑005).
3. **Selection:** select top‑k tiers, groups and experts deterministically (SPEC‑018).
4. **Pack tokens:** group tokens by their selected experts; prepare send buffers for each expert.
5. **Dispatch:** perform a deterministic all‑to‑all exchange of token buffers to the appropriate expert ranks (SPEC‑015).  Research shows that this all‑to‑all communication is the dominant cost in MoE training【529477976415087†L50-L63】.
6. **Expert compute:** on each expert rank, compute the feed‑forward network for the tokens it received.
7. **Unpack:** return the computed token outputs to their original order.
8. **Combine:** sum or concatenate expert outputs and apply any gating weights.
9. **Residual:** add the expert outputs back into the layer’s residual path.

## 2 Token Packing

Efficient packing is critical to reduce the number of messages sent.  The runtime maintains a mapping from tokens to experts and uses stable sorting to maintain determinism.  Pack buffers must be contiguous in memory to support DMA transfers.

## 3 Communication Scheduling

All‑to‑all exchanges are scheduled according to the deterministic collective protocol (SPEC‑018).  To overlap communication with computation, the runtime can initiate sends as soon as a token’s expert is known while continuing to compute routing scores for other tokens.

## 4 Parallelism Interaction

* **Data parallel:** tokens are distributed across data parallel ranks; each rank performs routing and dispatch only for its tokens.
* **Expert parallel:** experts are distributed across ranks; dispatch sends tokens to their corresponding expert rank.
* **Tensor parallel:** within a rank, expert computation may be split across multiple GPUs, requiring further local communication.

## 5 Latency Mitigation

To reduce latency:

* **Micro‑batching:** process multiple tokens per dispatch to amortise overhead.
* **Compression:** compress token representations before dispatch when network bandwidth is limited.
* **Load balancing:** ensure experts receive roughly equal numbers of tokens (SPEC‑032) to avoid stragglers.

By carefully structuring the token routing pipeline, Lite LLM achieves high throughput inference despite the complexity of hierarchical routing and distributed execution.
