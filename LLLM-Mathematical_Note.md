⸻

Mathematical Note: Stability, Capacity Bounds, and Deterministic Entropy in Lite LLM

A. Hierarchical Routing as Conditional Probability

Lite LLM’s HSER is most cleanly expressed as a hierarchical conditional distribution over the triple (t,g,e) (tier, group, expert), conditioned on a token state h := \mathbf{h}_i^{(l)}, TierSet \mathcal{T}, and seed s.

Let:
	•	t \in \mathcal{T}_{\max} be a tier id,
	•	g \in \{1,\dots,G_t\},
	•	e \in \{1,\dots,E_{t,g}\},
	•	\mathcal{T} \subseteq \mathcal{T}_{\max} the allowed tiers for the request.

Define logits:
z_t(h) := \langle w^{(l)}_{\text{tier},t}, h \rangle + b^{(l)}_{\text{tier},t},
\quad
z_{t,g}(h) := \langle w^{(l)}_{\text{group},t,g}, h \rangle,
\quad
z_{t,g,e}(h) := \langle w^{(l)}_{\text{expert},t,g,e}, h \rangle.

Define masked tier distribution (hard-zero outside TierSet):
p(t\mid h,\mathcal{T}) \;=\;
\frac{\mathbf{1}[t\in \mathcal{T}]\,\exp(z_t(h)/\tau_t)}
{\sum_{t'\in \mathcal{T}} \exp(z_{t'}(h)/\tau_t)}.

Group conditional:
p(g\mid t,h) \;=\;
\frac{\exp(z_{t,g}(h)/\tau_g)}
{\sum_{g'=1}^{G_t}\exp(z_{t,g'}(h)/\tau_g)}.

Expert conditional:
p(e\mid t,g,h) \;=\;
\frac{\exp(z_{t,g,e}(h)/\tau_e)}
{\sum_{e'=1}^{E_{t,g}}\exp(z_{t,g,e'}(h)/\tau_e)}.

Hierarchical joint:
p(t,g,e\mid h,\mathcal{T})
=
p(t\mid h,\mathcal{T})\,p(g\mid t,h)\,p(e\mid t,g,h).

This already encodes your key property:

If t\notin \mathcal{T}, then p(t,g,e\mid h,\mathcal{T}) = 0.
Quadrillion-scale banks impose zero compute unless explicitly allowed.

⸻

B. Deterministic Selection as Seeded ArgTopK on a Total Order

Training and inference use selection (Top-K) rather than sampling. Define a deterministic operator that maps real scores to an ordered selection.

Let an index set \Omega (tiers, groups, or experts). For each j\in\Omega, we have score x_j\in\mathbb{R}. Define a seeded tie-breaker:
\epsilon_j(s) := \mathrm{U32Hash}(j \oplus s) / 2^{32} \in [0,1).

Define a total order key:
\kappa_j := \big(-x_j,\, \epsilon_j(s),\, j\big)
and sort lexicographically. Then:

\mathrm{TopK}_{\text{stable}}(x,k,s) := \text{the first }k\text{ indices under the total order } \kappa.

Deterministic Entropy Definition (Lite LLM):
The only “entropy” admitted is the deterministic pseudo-randomness \epsilon_j(s), which is a pure function of (j,s). No runtime nondeterminism is permitted.

Bitwise reproducibility requirement: all ranks must:
	1.	compute identical x_j (or use quantized logits for routing),
	2.	compute identical hashes \epsilon_j(s),
	3.	sort using identical total ordering.

This removes “heisenbugs” from distributed MoE routing.

⸻

C. HSER Convergence and Anti-Collapse Guarantees

Winner-take-all (WTA) collapse is the principal failure mode at extreme capacity. Lite LLM prevents collapse by enforcing entropy + load constraints at each routing level.

C.1 Router Marginals
For a given layer l, over a batch \mathcal{B} of token states h, define marginal usage:

Tier marginal:
u_t := \mathbb{E}_{h\sim\mathcal{B}}\left[p(t\mid h,\mathcal{T})\right].

Group marginal (within tier):
u_{t,g} := \mathbb{E}_{h\sim\mathcal{B}}\left[p(g\mid t,h)\right].

Expert marginal (within tier+group):
u_{t,g,e} := \mathbb{E}_{h\sim\mathcal{B}}\left[p(e\mid t,g,h)\right].

Collapse manifests as high concentration (e.g., \max u_{t,g,e} large, entropy low).

C.2 Multi-level Auxiliary Loss (entropy + uniformity)
A clean, compositional penalty is a sum of KL-to-uniform terms:

Tier balancing (over allowed tiers \mathcal{T}):
\mathcal{L}_{\text{tier}} := \mathrm{KL}\!\left(u_{\mathcal{T}} \,\big\|\, \mathrm{Unif}(\mathcal{T})\right).

Group balancing (per tier):
\mathcal{L}_{\text{group}} := \sum_{t\in \mathcal{T}} \mathrm{KL}\!\left(u_{t,\cdot} \,\big\|\, \mathrm{Unif}(\{1,\dots,G_t\})\right).

Expert balancing (per tier+group):
\mathcal{L}_{\text{expert}} := \sum_{t\in \mathcal{T}} \sum_{g=1}^{G_t}
\mathrm{KL}\!\left(u_{t,g,\cdot} \,\big\|\, \mathrm{Unif}(\{1,\dots,E_{t,g}\})\right).

Total auxiliary:
\mathcal{L}_{\text{aux}} := \alpha_t \mathcal{L}_{\text{tier}} + \alpha_g \mathcal{L}_{\text{group}} + \alpha_e \mathcal{L}_{\text{expert}}.

This yields a precise “anti-WTA” statement:

Minimizing \mathcal{L}_{\text{aux}} forces marginals toward uniformity, preventing concentration collapse at each routing depth.

You can optionally replace KL with CV or Rényi entropy; KL-to-uniform is the cleanest for proofs.

C.3 Capacity–Load Feasibility Condition (non-collapse necessary condition)
Let N be tokens per step globally (after DP aggregation). Let K := k_{\text{tier}}k_gk_e be active experts per token.

Total expert-assignment events per step:
A = N\cdot K.

Suppose a tier contains M_t := \sum_g E_{t,g} experts. Under perfect balance within that tier, expected assignments per expert are \approx A_t / M_t. If A_t / M_t \ll 1, most experts receive no gradient update and will stagnate.

Necessary condition for “learning” a tier t:
\frac{A_t}{M_t} \ge \rho
\quad\text{for some minimum update density }\rho>0.

This is why Lite LLM treats quadrillion scale as a parameter universe: most experts are not meant to be trained simultaneously unless token throughput is enormous. Practically, you train the active frontier (hot tiers) and expand gradually.

This condition becomes a design knob:
	•	increase throughput N,
	•	increase activation K (careful),
	•	or restrict the trainable subset of experts in a tier (curriculum / gating freeze).

⸻

D. The Invariant: P_{\text{active}} \ll P_{\text{total}} with Scaling Benefits

D.1 Definitions
Let total parameter count:
P_{\text{total}} = P_{\text{dense}} + \sum_{t\in\mathcal{T}_{\max}} P^{\text{experts}}_t.

Let each selected expert have parameter size |\theta_{t,g,e}| bounded by \Theta_{\max} under config.

Let dense per-layer active parameters (attention+norm) be bounded by C_{\text{dense}} (a fixed function of d, heads, etc.).

Per token per layer:
P_{\text{active}}^{(l)} \le C_{\text{dense}} + K\cdot \Theta_{\max}.

Over L layers:
P_{\text{active}} \le L(C_{\text{dense}} + K\Theta_{\max}).

Theorem (Capacity Independence)
For fixed (L, C_{\text{dense}}, K, \Theta_{\max}),
P_{\text{active}} = O(1)\quad\text{with respect to}\quad P_{\text{total}}.
So P_{\text{active}} does not grow when you add tiers or experts, as long as TierSet and K remain fixed.

D.2 Scaling Benefit Without Dense Scaling
Define the model as a mixture family where additional experts increase representational capacity. If we consider the MoE layer output:
\mathrm{MoE}(h) = \sum_{(t,g,e)\in \mathcal{E}_{\text{active}}} \pi_{t,g,e}(h)\,\mathrm{Expert}_{t,g,e}(h),
with \sum \pi_{t,g,e}(h)=1 over the active set.

Adding experts increases the available hypothesis class even if only few are active per token, by enabling conditional specialization: the function class becomes a union/mixture of many local functions. This preserves scaling gains by expanding the number of specialized regions without increasing per-token compute (the router chooses which region is used).

Formally, the function family cardinality (informally) grows with number of experts while evaluation cost remains bounded by K.

⸻

E. Deterministic Distributed Training: “Deterministic Entropy” as a Contract

Lite LLM’s determinism contract can be stated as:

Determinism Contract. For each step index n, layer l, token index i, and routing level \lambda\in\{\text{tier},\text{group},\text{expert}\},
the selected index sets are functions:
\mathcal{S}_{\lambda}(n,l,i) = F_{\lambda}\big(h_i^{(l)}, W_{\lambda}^{(l)}, \mathcal{T}, s(n,l,i,\lambda)\big),
where s(\cdot) is computed identically across ranks and F_{\lambda} uses \mathrm{TopK}_{\text{stable}} with a total order.

This guarantees that:
	•	token packing to experts is identical,
	•	all-to-all layouts match,
	•	gradients aggregate consistently.

A practical strengthening (often needed at scale) is to route using quantized logits (e.g., int16 fixed point) to remove tiny floating-point divergence across devices; mathematically this replaces z with \lfloor \gamma z \rceil.

⸻

Summary: The Three Failure Points, Mathematically Addressed
	1.	HSER Convergence / Avoiding WTA
Multi-level marginal balancing via KL-to-uniform (or CV/entropy) keeps all routing levels from collapsing.
	2.	Invariant Proof P_{\text{active}}\ll P_{\text{total}}
Bounded Top-K at each level implies P_{\text{active}} \le L(C_{\text{dense}} + K\Theta_{\max}), independent of total tiers/expert banks.
	3.	Deterministic Entropy
Stable Top-K under a seeded total order yields bitwise-reproducible selection, eliminating nondeterminism in distributed MoE dispatch.

⸻
