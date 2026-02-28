# SPEC‑048 — Cost‑Adaptive Routing

While HSER selects experts based on hidden state, some experts may be expensive in terms of latency, memory or energy.  **Cost‑adaptive routing** augments the routing function to account for such costs and adhere to user‑specified budgets.

## 1 Cost Model

1. **Per‑expert cost:** Each expert `e` has an estimated cost vector `(c_latency, c_memory, c_energy)` reflecting the latency of loading its parameters, memory footprint and energy consumption (e.g., GPU utilization).  Costs may be learned from telemetry or provided by operators.
2. **Tier cost:** Each tier `t` has an aggregate cost, reflecting the cumulative impact of activating all experts in that tier.  Deeper tiers (e.g., cold or archive) typically have higher latency costs due to I/O【75891756086750†L80-L95】.
3. **Budget constraints:** A request may specify budgets `(B_latency, B_memory, B_energy)`.  These budgets define upper bounds on per‑token or per‑request cost.

## 2 Modified Routing Objective

The standard HSER routing maximises the expert scores `s_{t,g,e}(h)`.  Cost‑adaptive routing modifies the objective:

\[
\tilde{s}_{t,g,e}(h) = s_{t,g,e}(h) - \lambda \cdot \mathrm{Cost}_{t,g,e}
\]

where `λ` is a weight vector balancing performance and cost, and `Cost_{t,g,e}` is the normalised cost of selecting expert `e`.

* **Normalisation:** Costs are normalised across all candidate experts to ensure comparability.
* **Seeded tie‑breaking:** After modifying scores, stable top‑k selection with seeded tie‑breaking (SPEC‑003) is applied.
* **Budget enforcement:** If the sum of selected costs exceeds the budget, the router prunes the lowest‑score expert and selects the next candidate until the budget is respected.

## 3 Training Considerations

1. **Auxiliary cost loss:** A cost regularisation term may be added to the loss function to encourage the model to assign high probabilities to low‑cost experts when quality differences are small.
2. **Curriculum:** During early training, cost weights may be small to allow the model to learn expressive experts.  As training progresses, cost weights increase to enforce budget awareness.
3. **Expert specialisation:** High‑cost experts may specialise in complex patterns; cost‑adaptive routing encourages the router to consult them only when necessary.

## 4 Runtime Policy

1. **Static vs dynamic weights:** Administrators may set `λ` globally or per user.  For example, free users might have higher `λ_latency`, while premium users have lower cost weights.
2. **Budget solver:** The latency budget solver (SPEC‑042) integrates cost‑adaptive routing by adjusting `λ` such that the selected experts satisfy budgets.  The solver may re‑run the routing with updated weights if the initial selection violates the budget.
3. **Transparency:** The system can expose to users the trade‑off between quality and cost, enabling them to choose a `λ` that meets their needs.

## 5 Determinism

Cost‑adaptive routing preserves determinism because the modified scores and budgets are deterministic functions of the hidden state, seed and cost parameters.  Tie‑breaking and selection remain stable and reproducible.

By incorporating cost into the routing objective, Lite LLM respects latency and resource budgets while leveraging deep expert banks.  This allows the model to adapt to diverse deployment environments (edge devices, servers) without retraining.【75891756086750†L80-L95】
