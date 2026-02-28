# SPEC‑041 — TierSet Selection Engine

At inference time, the user or service must choose a **TierSet** — a subset of tiers that will be active for the request (e.g., `hot` only for fast responses, or `hot + warm` for more capacity).  The TierSet selection engine determines which tiers to enable based on policies, budgets and user inputs.

## 1 Inputs

* **User request:** may specify desired quality (e.g., `fast`, `balanced`, `max`) or explicitly list tiers.
* **Latency budget:** maximum allowable latency for the request.
* **Cost budget:** monetary cost or resource usage constraints.
* **Availability:** tiers that are currently loaded and ready; some may be unavailable due to maintenance or memory pressure.

## 2 Selection Policies

### 2.1 Fixed Modes

Predefined modes map to specific TierSets:

* **Fast mode:** hot tier only; lowest latency, smallest capacity.
* **Balanced mode:** hot + warm; modest latency and capacity.
* **Deep mode:** hot + warm + cold; higher latency but more capacity.
* **Max mode:** all available tiers; highest latency and cost but full capacity.

### 2.2 Budget‑Based Selection

Given a latency or cost budget, solve a small optimisation problem to select the TierSet that maximises predicted response quality while respecting the budget.  This can be formulated as a knapsack problem over tiers (SPEC‑042 provides details).

### 2.3 Dynamic Selection

Adjust the TierSet during a session based on observed latency and user feedback.  For example, start in fast mode and progressively enable additional tiers if the user requests more detail.

## 3 Implementation Steps

1. **Gather inputs:** parse user or application preferences and budgets.
2. **Filter tiers:** exclude tiers that are unavailable or violate security policies (SPEC‑055).
3. **Compute costs:** estimate latency and resource cost for each candidate TierSet using historical telemetry (SPEC‑049).
4. **Select:** choose a TierSet according to the policy; optionally solve the optimisation problem.
5. **Activate:** notify the runtime to activate the selected TierSet for the request.  Lazy loading will bring in experts as needed (SPEC‑026).

## 4 User Overrides and Safety

Users may explicitly request or forbid certain tiers.  Policies should honour these overrides unless they violate system constraints (e.g., security restrictions).  Administrators can define minimum and maximum TierSets for certain endpoints.

By implementing a TierSet selection engine, Lite LLM provides flexible inference quality options that align with performance and cost requirements.
