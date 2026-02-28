# Lite LLM Enterprise Runtime Specification 006: Active Compute Bounding Model

## Purpose

This specification formalizes the bounding model that limits the amount of computation per token in Lite LLM.  It derives FLOP counts, establishes latency scaling laws and demonstrates how the architecture remains feasible at quadrillion‑parameter scale.

## Formal Compute Bounds

Let $d$ be the hidden dimension, $L$ the number of transformer layers and $K = k_{\text{tier}}\times k_g\times k_e$ the number of active experts per token.  Assume each expert is a two‑layer MLP with hidden dimension $h$ and output dimension $d$; then the FLOP cost per expert invocation is approximately $2d\times h + h\times d = O(dh)$.  The per‑token cost across all experts in a layer is $K\times O(dh)$.

For dense components (attention, RMSNorm), the per‑token cost per layer is $O(d^2)$.  Summing over $L$ layers gives

\[
\mathrm{FLOPs}_{\text{token}} \approx L\Bigl[ O(d^2) + K\,O(dh) \Bigr].
\]

Because $K$ is constant and small, increasing the number of experts or adding new tiers does not change this formula.  Instead, the capacity of the model grows by increasing the number of experts (the count of parameter matrices), which are sparsely activated.

## Latency Scaling Laws

The end‑to‑end latency for a request includes computation, communication and I/O:

* **Computation:** Scales linearly with the number of layers $L$ and constant factors from $K$ and $d$.
* **Communication:** In expert parallelism, each token is dispatched to its selected experts via an all‑to‑all exchange.  The expected communication volume per rank is $O\bigl(\frac{N L K d}{R_{\text{EP}}}\bigr)$, where $N$ is the batch size and $R_{\text{EP}}$ is the number of expert‑parallel ranks.  See SPEC 015 for details.
* **I/O:** When tiers reside in cold storage, additional latency arises from loading experts.  Intelligent prefetching (SPEC 045) and caching policies (SPEC 022–024) minimize these costs.

Empirically, latency is dominated by communication for large $d$ and by I/O when cold tiers are frequently accessed.  Tuning TierSets to match latency budgets (SPEC 042) is key.

## Quadrillion‑Parameter Scaling Math

Suppose each expert contains $\Theta_{\max}=8$ million parameters and $K=4$.  A model with 1 quadrillion parameters can be realized by adding 125 million such experts across many tiers.  However, per‑token compute remains bounded by $4 \times 8\text{M}$ parameters per layer.  With 128 layers, this is roughly 4.1 billion active parameters per token—consistent with the bounding formula.  Memory bandwidth and communication become the limiting factors rather than arithmetic.

## Implementation Considerations

* **Batch Size:** Increasing batch size $N$ improves throughput but requires more memory and communication.  Expert parallel partitions should be sized to keep per‑rank token counts manageable.
* **Hidden Dimension Scaling:** Doubling $d$ roughly quadruples attention cost ($d^2$) and doubles expert cost ($d h$).  Choose $d$ consistent with hardware capabilities.
* **Expert Size:** The hidden dimension of experts $h$ is often a multiple of $d$.  Larger $h$ increases model capacity linearly but also increases compute.

## References

For related scaling law analysis, consult the GShard and Switch Transformer papers listed in **References.md**.