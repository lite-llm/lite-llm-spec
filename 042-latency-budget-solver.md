# SPEC‑042 — Latency Budget Solver

Inference latency depends on the number of active experts, their placement and network bandwidth.  Given a latency budget, Lite LLM must determine the largest TierSet that can be enabled without exceeding the budget.  This specification defines a simple solver for selecting tiers under latency constraints.

## 1 Latency Model

Total latency is approximated as the sum of:

* **Base latency (L_base):** time to compute the dense backbone (embeddings, attention) for a given context length.
* **Expert latency:** sum of latencies for each active expert.  An expert’s latency includes compute and data transfer time; transfers from the warm or cold tiers introduce additional delay.
* **Communication latency:** overhead for routing consensus and all‑to‑all dispatch (SPEC‑015).  The RailS study shows that all‑to‑all communication dominates MoE iteration time and load balancing is crucial【529477976415087†L50-L63】.

To simplify, define average per‑expert latency values for each tier: `lat_hot`, `lat_warm`, `lat_cold`, `lat_archive`.  Multiply by the number of experts activated from that tier.

## 2 Optimisation Problem

Given:

* Latency budget `L_budget`.
* Per‑tier latency values.
* Number of experts to activate (`k_tier`, `k_g`, `k_e`).
* TierSet candidate list.

Find the TierSet that maximises capacity (sum of parameter budgets of selected tiers) subject to:

```
L_base + Σ (num_experts_tier × lat_tier) ≤ L_budget
```

This is a knapsack‑like optimisation: each tier has a cost (latency) and a value (capacity).  With a small number of tiers, a brute‑force search over subsets is feasible.  For more tiers, a greedy algorithm based on value‑to‑cost ratio works well.

## 3 Solver Implementation

1. **Precompute costs and values:** for each tier, compute latency cost = `k_tier × k_g × k_e × lat_tier` and value = parameter budget.
2. **Enumerate subsets:** generate all TierSet candidates consistent with user and system constraints.
3. **Select:** pick the TierSet with maximum value whose total cost plus base latency is ≤ `L_budget`.
4. **Fallback:** if no non‑empty TierSet meets the budget, select the minimal TierSet (e.g., hot only) and warn that latency may exceed the budget.

## 4 Adaptive Policies

During inference, actual latencies may differ from estimates.  The solver can incorporate feedback from telemetry (SPEC‑049) to update per‑tier latency estimates.  If a request consistently misses its budget, reduce the TierSet in subsequent requests.

By solving a simple optimisation problem, the latency budget solver balances capacity and responsiveness, enabling Lite LLM to meet service‑level objectives.
