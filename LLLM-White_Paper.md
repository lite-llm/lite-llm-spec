# **Lite LLM**

## **A Deterministic Tiered Large Language Model Architecture Scaling from 1B to 1Q Parameters**

---

Author: Dust LLC
Date: February 2026
Status: Foundational White Paper

---

## Abstract

**Lite LLM** is a Rust-native large language model architecture designed to scale from **1 billion (10⁹)** to **1 quadrillion (10¹⁵)** parameters while maintaining **bounded active compute per token**.

Unlike monolithic dense transformer models, Lite LLM introduces two structural primitives:

1. **Tiered Parameter Architecture (TPA)** — capacity is organized into explicit parameter tiers (1B, 10B, 100B, 1T, …).
2. **Hierarchical Sparse Expert Routing (HSER)** — deterministic multi-level routing ensures that only a bounded subset of experts activates per token.

Lite LLM decouples **total parameter capacity** from **per-token compute**, enabling quadrillion-scale parameter banks without quadrillion-scale compute. The architecture is deterministic, checkpoint-compatible under expansion, storage-aware, and implemented entirely in Rust with explicit memory and routing guarantees.

---

# 1. Introduction

Scaling large language models traditionally increases dense parameter count, which increases:

* Memory linearly
* Compute quadratically (attention)
* Training cost superlinearly

This approach becomes infeasible beyond trillion scale.

Lite LLM instead establishes:

> **Total capacity may grow arbitrarily; active compute remains bounded.**

This is achieved through:

* Structured tier partitioning
* Hierarchical sparse routing
* Deterministic selection policies
* Tier-aware storage placement

---

# 2. Design Principles

### P1. Bounded Active Compute

For any request:

[
P_{\text{active}} \ll P_{\text{total}}
]

Active parameters per token do not scale with total model capacity.

---

### P2. Deterministic Routing

Routing is a pure function:

[
R(h, W, \mathcal{T}, s)
]

where:

* ( h ) = hidden state
* ( W ) = router weights
* ( \mathcal{T} ) = TierSet
* ( s ) = deterministic seed

No nondeterministic operations permitted.

---

### P3. First-Class Tiering

Tiering is structural:

* Encoded in checkpoint format
* Explicit in routing
* Explicit in storage placement
* Explicit in runtime policy

---

### P4. Expandability

New tiers may be added without retraining existing tiers and without invalidating previous checkpoints.

---

### P5. Rust-Native Implementation

All core logic implemented in safe Rust:

* Explicit ownership
* Trait-based device abstraction
* Deterministic collectives
* Unsafe confined to kernel backends

---

# 3. Model Architecture

## 3.1 Backbone Transformer

Lite LLM uses a standard transformer backbone:

Each layer ( l ):

1. RMSNorm
2. Multi-head attention
3. Residual
4. RMSNorm
5. MoE feedforward (tier-aware)
6. Residual

Hidden state:

[
\mathbf{h}_i^{(l)} \in \mathbb{R}^d
]

---

# 4. Tiered Parameter Architecture (TPA)

## 4.1 Tier Definition

Let:

[
\mathcal{T}_{\max} = { t_1, t_2, \dots, t_M }
]

Each tier ( t ) defines:

* Parameter budget ( B_t )
* Expert banks
* Placement policy
* Activation eligibility

Examples:

| Tier      | Target Capacity   |
| --------- | ----------------- |
| tier_1b   | 1B parameters     |
| tier_10b  | 10B parameters    |
| tier_100b | 100B parameters   |
| tier_1t   | 1T parameters     |
| tier_1q   | 1Q parameter bank |

---

## 4.2 TierSet

For each request:

[
\mathcal{T} \subseteq \mathcal{T}_{\max}
]

TierSet determines which tiers may be activated.

Modes:

* 1B Mode → {tier_1b}
* 10B Mode → {tier_10b}
* 100B Mode → {tier_100b}
* Max Mode → {tier_1t, tier_10t, …}

TierSet may be:

* Exclusive
* Cumulative
* Budget-constrained

---

# 5. Hierarchical Sparse Expert Routing (HSER)

## 5.1 Structure

For each tier ( t ):

* ( G_t ) groups
* ( E_{t,g} ) experts per group

Each expert:

[
\mathrm{Expert}_{t,g,e}: \mathbb{R}^d \to \mathbb{R}^d
]

---

## 5.2 Routing Levels

### Level 1 — Tier Router

[
\mathbf{s}*{\text{tier}}^{(l)} =
W*{\text{tier}}^{(l)} \mathbf{h}_i^{(l)}
]

Mask to ( \mathcal{T} ), apply softmax:

[
S =
\mathrm{TopK}*{\text{stable}}(
\mathrm{softmax}(\mathbf{s}*{\text{tier}}^{(l)}|*{\mathcal{T}}),
k*{\text{tier}},
s)
]

---

### Level 2 — Group Router

For ( t \in S ):

[
\mathbf{s}*{\text{group},t}^{(l)} =
W*{\text{group},t}^{(l)} \mathbf{h}_i^{(l)}
]

[
G_t^s =
\mathrm{TopK}*{\text{stable}}(
\mathrm{softmax}(\mathbf{s}*{\text{group},t}^{(l)}),
k_g,
s \oplus \mathrm{id}(t))
]

---

### Level 3 — Expert Router

[
\mathbf{s}*{\text{expert},t,g}^{(l)} =
W*{\text{expert},t,g}^{(l)} \mathbf{h}_i^{(l)}
]

[
E_{t,g}^s =
\mathrm{TopK}*{\text{stable}}(
\mathrm{softmax}(\mathbf{s}*{\text{expert},t,g}^{(l)}),
k_e,
s \oplus \mathrm{id}(t) \oplus g)
]

---

## 5.3 Active Expert Set

[
\mathcal{E}*{\text{active}}^{(l)} =
\bigcup*{t \in S}
\bigcup_{g \in G_t^s}
E_{t,g}^s
]

Cardinality:

[
|\mathcal{E}*{\text{active}}^{(l)}|
= k*{\text{tier}} \cdot k_g \cdot k_e
]

Independent of total tier count.

---

# 6. Core Theorem — Bounded Active Compute

Let:

[
P_{\text{total}} =
\sum_{t \in \mathcal{T}_{\max}} B_t
]

Let:

[
P_{\text{active}}^{(l)}
=======================

C_{\text{dense}} +
\sum_{e \in \mathcal{E}_{\text{active}}^{(l)}} |e|
]

Then:

[
P_{\text{active}}
\le
L
\left(
C_{\text{dense}}
+
k_{\text{tier}} k_g k_e
\cdot
\max_{t,g,e} |e|
\right)
]

This bound is independent of ( P_{\text{total}} ).

---

## Corollary — Quadrillion-Scale Feasibility

If:

* ( k_{\text{tier}} = 1 )
* ( k_g = 2 )
* ( k_e = 2 )
* Expert size = 8M parameters
* ( L = 128 )

Then:

[
P_{\text{active}} \approx 4.1 \text{B per token}
]

Even if:

[
P_{\text{total}} = 10^{15}
]

Active compute remains bounded.

---

# 7. Tier Expansion Operator

Adding new tier ( t_{\text{new}} ):

1. Freeze existing tiers
2. Initialize new experts and router heads
3. Train new parameters
4. Update checkpoint manifest

---

## Theorem — Backward Compatibility

Any checkpoint valid for TierSet ( \mathcal{T} ) remains valid for any superset ( \mathcal{T}' \supseteq \mathcal{T} ).

Proof:

* Routing mask ignores absent tiers.
* Checkpoints are tier-indexed.
* Existing weights remain untouched.

---

# 8. Storage Tiering

Each tier assigned placement:

[
P_t \in
{ \text{HBM}, \text{DRAM}, \text{NVMe}, \text{Object} }
]

Expected activation:

[
\pi_t =
\mathbb{E}[p_t(h)]
]

Prefetch priority:

[
\text{priority}(t)
==================

\pi_t \cdot \text{size}(t)
]

Runtime promotes tiers asynchronously.

---

# 9. Auxiliary Load-Balancing Loss

To prevent expert collapse:

[
\mathcal{L}_{\text{aux}}
========================

\sum_{l}
\sum_{\text{levels}}
\alpha
\cdot
\mathrm{CV}(p)
]

where:

[
\mathrm{CV}(p)
==============

\frac{\mathrm{std}(p)}{\mathrm{mean}(p)}
]

---

# 10. Budget-Constrained Tier Selection

Given:

* Latency limit ( \tau )
* Memory limit ( \mu )

Solve:

[
\max_{\mathcal{T}}
\sum_{t \in \mathcal{T}} U_t
]

subject to:

[
\sum c_t \le \tau
]

[
\sum m_t \le \mu
]

Small knapsack; greedy approximation sufficient.

---

# 11. Deterministic Guarantee

Routing uses:

* Stable sorting
* Seeded tie-breaking
* No nondeterministic reductions

Therefore:

[
R(h, W, \mathcal{T}, s)
]

is deterministic across machines.

---

# 12. Conclusion

Lite LLM establishes:

* Tiered parameter architecture
* Deterministic hierarchical routing
* Bounded active compute
* Quadrillion-scale parameter capacity
* Expandable checkpoints
* Storage-aware execution
* Rust-native safety

Lite LLM is not a larger transformer.
It is a **capacity-partitioned parameter universe with bounded activation and deterministic expansion**.

---
