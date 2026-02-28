---

# Lite LLM  
## A Deterministic Tiered Large Language Model Architecture Scaling from 1B to 1Q Parameters

---

Author: Dust LLC

Date: February 2026

Status: Foundational White Paper

---

## Abstract

Lite LLM is a Rust-native large language model architecture designed to scale from 1 billion (10^9) to 1 quadrillion (10^15) parameters while maintaining bounded active compute per token. Unlike dense transformer scaling, Lite LLM introduces Tiered Parameter Architecture (TPA) and Hierarchical Sparse Expert Routing (HSER) as first-class structural primitives.

The architecture decouples total parameter capacity from per-token compute cost. Only a fixed, bounded subset of experts is activated per token, independent of the total parameter universe. Lite LLM is deterministic by construction, storage-aware, checkpoint-compatible under expansion, and designed for safe implementation in Rust.

---

# 1. Introduction

Traditional transformer scaling increases dense parameter count, resulting in:

- Linear memory growth  
- Quadratic attention cost  
- Exponential infrastructure pressure  

Lite LLM reframes scaling as parameter universe management rather than dense width growth. Total capacity may grow arbitrarily large while active compute remains bounded.

The central invariant:

P_active << P_total

This enables quadrillion-scale parameter banks without quadrillion-scale compute.

---

# 2. Design Principles

## 2.1 Bounded Active Compute

Active parameters per token are independent of total capacity.

## 2.2 Deterministic Routing

Routing is a pure function:

R(h, W, T, s)

where:
- h = hidden state  
- W = router weights  
- T = TierSet  
- s = deterministic seed  

## 2.3 Tiering as a Structural Primitive

Tiering is encoded in:
- Routing  
- Checkpoint format  
- Storage placement  
- Runtime activation policy  

## 2.4 Expandability

New parameter tiers may be added without invalidating previous checkpoints.

## 2.5 Rust-Native Correctness

All core logic is implemented in safe Rust. Unsafe code is isolated to kernel backends.

---

# 3. Model Architecture

## 3.1 Backbone Transformer

Each layer l computes:

1. RMSNorm  
2. Multi-head attention  
3. Residual connection  
4. RMSNorm  
5. Tier-aware MoE feedforward  
6. Residual connection  

Hidden state at token i:

h_i^(l) ∈ R^d

---

# 4. Tiered Parameter Architecture (TPA)

## 4.1 Tier Definition

Let T_max = {t1, t2, ..., tM} be all available tiers.

Each tier t defines:

- Parameter budget B_t  
- Expert banks  
- Placement policy  
- Activation rules  

Example tiers:

- tier_1b  
- tier_10b  
- tier_100b  
- tier_1t  
- tier_1q  

Total parameters:

P_total = P_dense + Σ B_t

---

## 4.2 TierSet

For a given request:

T ⊆ T_max

TierSet determines which tiers may activate.

Examples:

- 1B mode → {tier_1b}  
- 10B mode → {tier_10b}  
- Max mode → {tier_1t, tier_10t, ...}

---

# 5. Hierarchical Sparse Expert Routing (HSER)

For tier t:

- G_t groups  
- E_t,g experts per group  

Each expert:

Expert_t,g,e : R^d → R^d

---

## 5.1 Hierarchical Conditional Probability

Routing is expressed as:

p(t, g, e | h, T)  
= p(t | h, T) · p(g | t, h) · p(e | t, g, h)

If t ∉ T, then p(t, g, e | h, T) = 0.

---

## 5.2 Deterministic Top-K Selection

Selection uses stable total ordering:

1. Sort by score descending  
2. Break ties via seeded hash  
3. Deterministic lexicographic order  

This guarantees reproducible routing across distributed ranks.

---

## 5.3 Active Expert Set

For configuration:

k_tier  
k_g  
k_e  

Active experts per layer:

|E_active| = k_tier × k_g × k_e

Independent of total expert count.

---

# 6. Core Theorem — Bounded Active Compute

Let:

C_dense = dense attention + normalization cost  
Θ_max = maximum expert parameter size  
L = number of layers  

Then:

P_active ≤ L ( C_dense + k_tier k_g k_e Θ_max )

Thus:

∂P_active / ∂P_total = 0

Active compute is independent of total capacity.

---

# 7. Communication Complexity

Let:

N = tokens per step  
d = hidden dimension  
K = k_tier k_g k_e  
R_EP = expert parallel ranks  

Total dispatch events per step:

A = N L K

Per-rank communication volume:

V_r = (N L K d) / R_EP

Thus communication scales with active experts, not total parameter count.

---

# 8. No-Starvation Lemma

Let:

M_t = experts in tier t  
A_t = assignments to tier t per step  

Expected updates per expert:

E[X_e] = A_t / M_t

If:

A_t / M_t ≥ ρ > 0

Then starvation probability over T steps:

Pr(X_e = 0) ≤ exp(-Tρ)

Thus starvation probability decays exponentially.

---

# 9. Load-Balancing Auxiliary Loss

To prevent routing collapse:

L_aux = Σ KL(u || Uniform)

applied at:

- Tier level  
- Group level  
- Expert level  

Ensures distributional spread.

---

# 10. Storage Tiering

Each tier assigned placement:

- HBM  
- DRAM  
- NVMe  
- Object store  

Expected activation:

π_t = E[p_t(h)]

Prefetch priority:

priority(t) = π_t × size(t)

---

# 11. Tier Expansion Operator

To add new tier t_new:

1. Freeze existing tiers  
2. Initialize new experts  
3. Train new routing heads  
4. Update checkpoint manifest  

Backward compatibility preserved.

---

# 12. Deterministic Entropy Contract

Routing selection is a function:

S = TopK_stable(scores, k, seed)

Seeded tie-breaking ensures identical ordering across machines.

Floating-point nondeterminism can be removed via quantized routing logits.

---

# 13. Quadrillion-Scale Feasibility

Example:

k_tier = 1  
k_g = 2  
k_e = 2  
Θ_max = 8M parameters  
L = 128  

P_active ≈ 4B per token  

Even if:

P_total = 10^15

Active compute remains bounded.

---

# 14. Conclusion

Lite LLM introduces:

- Tiered Parameter Architecture  
- Deterministic Hierarchical Sparse Expert Routing  
- Bounded active compute independent of total capacity  
- Expert parallel communication scaling  
- No-starvation guarantees  
- Storage-aware execution  
- Rust-native safety  

Lite LLM is not a larger transformer.

It is a structured parameter universe with deterministic activation and bounded computation, enabling practical trillion-scale systems and structurally feasible quadrillion-scale capacity.

---

References are provided in References.md.
