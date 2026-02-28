# Lite LLM Enterprise Runtime Specification 003: Deterministic Routing Engine

## Purpose

This specification details the deterministic routing engine responsible for mapping token hidden states to expert selections.  It defines the stable top‑$k$ algorithm, seed handling, tie‑breaking guarantees, floating‑point determinism constraints and cross‑node reproducibility requirements.

## Stable Top‑$k$ Algorithm

Given a score vector $x$ of length $n$ and a parameter $k$, the stable top‑$k$ operation returns $k$ indices with highest scores.  To ensure determinism across distributed environments:

1. **Quantization:** Router logits may be quantized to a fixed‑point representation before sorting to remove nondeterministic differences in floating‑point arithmetic across devices.
2. **Seeded Tiebreaking:** A deterministic hash $h(i, s)$ is computed from the index $i$ and global seed $s$.  Ties in score are broken first by descending order of $x$, then by ascending $h(i, s)$, then by index.  This creates a total order for selection.
3. **Stable Selection:** The algorithm must be stable—sorting equal elements preserves their original relative order after tie‑breaking.  Implementations may use priority queues or partial sorting; however, they must produce identical outputs given identical inputs and seeds.

## Seed Handling

The routing engine uses a 128‑bit global seed that is split deterministically across layers and tokens.  Seed derivation follows:

* **Base Seed:** A user‑defined or random 128‑bit value loaded at startup.
* **Layer Seed:** $s_l = \mathrm{Hash}(\text{base seed}, l)$ for layer $l$.
* **Token Seed:** $s_{l,i} = \mathrm{Hash}(s_l, i)$ for token $i$ at layer $l$.

Deterministic hash functions must be portable across programming languages and platforms.  FNV‑1a or xxHash with fixed constants are typical choices.

## Floating‑Point Determinism Constraints

The routing computation involves dot products between hidden states and router weights.  To guarantee reproducibility:

* **Deterministic Math Libraries:** Use deterministic BLAS or custom implementations that produce identical results across devices.  Avoid vendor‑specific fused kernels that may change accumulation order.
* **Fixed‑Point Pre‑Sorting:** Convert scores to a fixed‑point representation before top‑$k$ to mitigate minute differences in floating‑point rounding.
* **Seeded Noise:** If noisy routing is used for exploration, the noise must be generated deterministically using the token seed.

## Cross‑Node Routing Reproducibility

In distributed training and inference, all nodes must agree on routing decisions for correct all‑to‑all communication.  The engine ensures reproducibility by:

* **Broadcasting Seeds:** The base seed and layer seeds are broadcast to all nodes at initialization.
* **Synchronizing Hidden States:** Tensor parallel partitions share hidden states through deterministic collectives before routing.  Each node computes identical score vectors.
* **Selecting Routes Locally:** After synchronization, each node performs the same stable top‑$k$ selection, producing identical assignments.
* **Verifying Consistency:** Debug builds may optionally exchange routing indices to verify consistency; mismatches are logged and trigger fail‑fast behaviour.

## Tie‑Breaking Guarantees

The deterministic hash ensures that identical scores are resolved consistently.  The hash domain must be large enough to avoid collisions in practical scenarios.  In addition, the selection must be deterministic across machines with different endianess or integer sizes.  All arithmetic is performed in a standardized bit width (e.g., 64‑bit unsigned integers).

## References

For related work on deterministic routing and stable sorting, see the literature on Switch Transformers and GShard.  Further details can be found in **References.md**.