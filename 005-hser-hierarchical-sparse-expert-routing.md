# Lite LLM Enterprise Runtime Specification 005: Hierarchical Sparse Expert Routing (HSER)

## Purpose

This specification formalizes the Hierarchical Sparse Expert Routing (HSER) mechanism used in Lite LLM.  HSER deterministically maps each token’s hidden state to a small set of experts via tier, group and expert gates.  It provides mathematical bounds on expert activation and describes load‑balancing mechanisms.

## Tier → Group → Expert Gating

### Tier Selection

For each token, the tier router computes scores $z_t$ for each tier $t$ in the tier universe.  After masking out tiers not in the active TierSet, it applies a softmax to obtain probabilities $p(t|h)$ and selects the top $k_{\text{tier}}$ tiers using the deterministic top‑$k$ operator described in SPEC 003.  Selected tiers form the set $S$.

### Group Selection

For each selected tier $t\in S$, the group router computes scores $z_{t,g}$ over groups $g\in\{1,…,G_t\}$.  A softmax produces $p(g|t,h)$ and the top $k_g$ groups are chosen.  The selected group set is $G_t^s$.

### Expert Selection

For each selected group $(t,g)$, the expert router computes scores $z_{t,g,e}$ over experts $e\in\{1,…,E_{t,g}\}$.  After softmax, the top $k_e$ experts are selected.  The resulting active expert set is

\[
\mathcal{E}_{\text{active}} = \bigcup_{t\in S}\bigcup_{g\in G_t^s}E_{t,g}^s.
\]

The total number of active experts per token is $K = k_{\text{tier}}\times k_g\times k_e$.

## Load Balancing Math

Unbalanced routing can lead to expert collapse, where a few experts receive all traffic.  HSER uses auxiliary losses at each routing level to encourage uniform usage.  For a batch of tokens, define the marginal distributions $u_t$, $u_{t,g}$ and $u_{t,g,e}$ for tiers, groups and experts, respectively.  Load‑balancing loss is

\[
\mathcal{L}_{\text{LB}} = \alpha_{\text{tier}}\,\mathrm{KL}(u_\mathcal{T} \Vert \mathrm{Unif}(\mathcal{T})) + \alpha_{\text{group}}\,\sum_{t} \mathrm{KL}(u_{t,\cdot}\Vert \mathrm{Unif}(G_t)) + \alpha_{\text{expert}}\,\sum_{t}\sum_{g} \mathrm{KL}(u_{t,g,\cdot}\Vert \mathrm{Unif}(E_{t,g})).
\]

Hyperparameters $\alpha_{\text{tier}}, \alpha_{\text{group}}, \alpha_{\text{expert}}$ control the strength of balancing at each level.  Alternative metrics like the coefficient of variation can be used.

## Expert Activation Bounds

Given $K$ active experts per token and $L$ transformer layers, the maximum number of expert invocations per token is $L\times K$.  Each expert invocation processes a vector of dimension $d$ and performs $O(d\times h)$ computations, where $h$ is the hidden dimension of the expert.  Since $K$ is fixed and small, the activation cost grows linearly with the number of layers but remains independent of the total number of experts.

## Formal Compute Invariants

HSER upholds the invariant that the active parameter count per token does not grow with total capacity.  The bounded compute formula specified in SPEC 001 derives directly from HSER’s gating design.  The deterministic nature of HSER also ensures that identical tokens processed on different machines select the same experts, enabling consistent distributed execution.

## References

HSER builds upon mixture‑of‑experts literature such as Switch Transformers, GShard and Expert Choice routing.  See **References.md** for additional reading.