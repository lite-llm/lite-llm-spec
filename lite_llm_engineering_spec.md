# Lite LLM Engineering Specification

## 1 Introduction

Modern language models demonstrate strong performance improvements as the number of parameters increases, but the traditional approach—adding width and depth to dense transformers—causes the memory and compute cost to grow prohibitively.  When every token activates *all* parameters, even a 1‑trillion‑parameter model demands enormous memory bandwidth and energy.  Sparse Mixture‑of‑Experts (MoE) architectures were revived to address this bottleneck.  Instead of executing all experts, MoE uses a gating network to select a small subset of experts for each token, enabling **sparse activation** while maintaining a large pool of parameters.  As the literature notes, MoE architectures maintain a large parameter count but “activate only a small fraction of them per input,” keeping computation in check while benefiting from a huge pool of specialized knowledge【520653549415222†L58-L67】.  Top‑$k$ gating selects one or a few experts for each token and ignores the rest, which means increasing the number of experts does not proportionally increase computation【520653549415222†L117-L124】.

Lite LLM extends this idea to the extreme: it introduces a **Tiered Parameter Architecture (TPA)** and **Hierarchical Sparse Expert Routing (HSER)** to scale from 1 billion (1 B) to 1 quadrillion (1 Q) parameters.  TPA explicitly partitions parameters into named tiers (1 B, 10 B, 100 B, 1 T, …), allowing the model to operate in different capacity modes.  HSER performs deterministic, multi‑level gating (tier → group → expert) so that only a fixed number of experts are activated per token, independently of the total parameter count.  All core logic is written in Rust, leveraging memory‑safe language guarantees.  A 2025 U.S. cybersecurity report highlights that memory‑safe languages provide built‑in safeguards and shift safety burdens from developers to the language itself【443264331305965†L32-L37】; such languages (including Rust) embed protections against buffer overflows and other memory vulnerabilities【443264331305965†L73-L78】, making them a strategic choice for secure, high‑reliability systems.

## 2 Design Principles

Lite LLM’s design is governed by five non‑negotiable principles:

1. **Bounded Active Compute.** For any request, the number of active parameters remains bounded and does not scale with the total parameter count.  In MoE systems this is achieved by selecting only the top‑$k$ experts per token【520653549415222†L117-L124】; Lite LLM generalizes this bound through tier‑aware gating.

2. **Deterministic Routing.** All routing decisions are a pure function of the token’s hidden state, the gating weights, a configurable seed and the allowed TierSet.  The gating network outputs a score for each tier, group and expert; a stable top‑$k$ operator with a seeded tie‑breaker yields bitwise‑reproducible selections.  Determinism is essential to avoid nondeterministic training bugs and to ensure reproducible model evaluation.

3. **First‑Class Tiering.** Tiers are not a runtime hack but a structural part of the model.  Each tier has its own parameter budget, placement policy and routing heads.  The TierSet can be specified by the user (e.g., run in 10 B mode) or selected automatically based on latency/memory budgets.

4. **Expandability.** Capacity can be expanded by adding new tiers without retraining the entire model.  Checkpoints are tier‑indexed and maintain backward compatibility: a checkpoint trained on tiers {1 B, 10 B} remains valid when a new 100 B tier is added.

5. **Rust‑Native Safety.** The core implementation is in Rust.  Memory‑safe languages like Rust embed protections against buffer overflows and dangling pointers【443264331305965†L73-L78】, shifting safety burdens from developers to the compiler and language runtime【443264331305965†L32-L37】.  This reduces the risk of memory corruption when managing billions to quadrillions of parameters across heterogeneous memory.

## 3 Architecture Overview

### 3.1 Model Backbone

Lite LLM builds upon the transformer architecture but replaces each dense feed‑forward (FFN) layer with a tier‑aware MoE layer.  Each transformer block consists of:

1. RMSNorm on the hidden state.
2. Multi‑head self‑attention.  For long context lengths, rotary positional encoding (RoPE) is applied.
3. Residual connection.
4. RMSNorm.
5. Tier‑aware MoE feed‑forward.  Instead of a single dense FFN, the hidden state is routed to one or more experts selected via HSER.  Each expert is a small MLP.  The outputs of selected experts are combined (e.g., summed or averaged), weighted by their routing probabilities.
6. Residual connection.

This design maintains the expressive power of transformers while offloading most parameters into sparsely activated experts.

### 3.2 Tiered Parameter Architecture (TPA)

Parameters are partitioned into discrete **tiers**, each named by its approximate total capacity (e.g., *tier_1b*, *tier_10b*, *tier_100b*, *tier_1t*, etc.).  Each tier defines:

* **Parameter budget** – the sum of all expert parameters in the tier.
* **Expert groups** – within a tier, experts are organized into groups; each group contains multiple experts (feed‑forward MLPs).  This grouping allows hierarchical routing and scalable sharding across devices.
* **Placement policy** – a hint indicating where parameters should reside (hot GPU memory, warm CPU memory, cold NVMe, or archival object storage).  The StrataServe system demonstrates how a hierarchical parameter server can coordinate GPU high‑bandwidth memory (HBM), CPU DRAM and SSD storage to support sparsity‑dominated deep models.  It caches only the working parameters in GPU memory, uses direct GPU‑to‑GPU communication for intra‑node transfers and overlaps network, disk and compute in a four‑stage pipeline【696435369415399†L100-L115】.  This memory‑hierarchy design guides Lite LLM’s runtime: hot tiers reside in HBM, warm tiers in DRAM, cold tiers in NVMe and rarely accessed tiers in remote object storage.

A **TierSet** is the collection of tiers enabled for a particular inference or training run.  For example, running in “10 B mode” might enable `{tier_10b}` (exclusive) or `{tier_1b, tier_10b}` (cumulative).  TierSets allow the system to meet latency or memory budgets by restricting which tiers can be activated.

### 3.3 Hierarchical Sparse Expert Routing (HSER)

HSER deterministically selects a fixed number of experts for each token through three routing levels:

1. **Tier Router.** For a hidden state $h$ at layer $l$, a tier gate produces scores $z_t$ for each tier in the global tier universe.  These scores are masked to the current TierSet; a softmax produces a probability distribution over allowed tiers.  A stable top‑$k$ operator picks $k_{\text{tier}}$ tiers.  The selection is seeded, ensuring reproducibility.

2. **Group Router.** Within each selected tier $t$, another gate computes scores $z_{t,g}$ for each group $g$.  A softmax followed by top‑$k$ picks $k_g$ groups per tier.

3. **Expert Router.** Within each selected group $(t,g)$, a third gate scores each expert $e$.  A softmax and top‑$k$ picks $k_e$ experts.  The active expert set for the token is the cross product of tier, group and expert selections.  The total number of active experts per token is $K = k_{\text{tier}} \times k_g \times k_e$, which remains small (e.g., $1 \times 2 \times 2 = 4$) even if there are millions of experts in total.

The routing uses **sparse gating** rather than dense gating.  Early MoE models used dense gating (a weighted sum of all experts), but this is computationally expensive【520653549415222†L112-L117】.  Modern MoEs, and Lite LLM, perform *top‑$k$* gating: only the highest‑scoring experts are executed【520653549415222†L117-L124】.  This ensures that increasing the number of experts does not increase per‑token computation.  The gating network may add small noise to break ties (noisy top‑$k$), but Lite LLM uses a seeded hash for tie‑breaking to guarantee determinism.

### 3.4 Bounding Active Compute

Let $L$ be the number of transformer layers and $\Theta_{\max}$ the maximum expert parameter size (number of parameters in a single expert).  Each layer activates at most $K$ experts per token.  The total active parameters per token across all layers is bounded by

\[
P_{\text{active}} \le L \bigl(C_{\text{dense}} + K \cdot \Theta_{\max}\bigr),
\]

where $C_{\text{dense}}$ is the cost of dense components (attention, normalization).  Crucially, $P_{\text{active}}$ is independent of the total number of tiers or experts; adding a new tier increases $P_{\text{total}}$ but not $P_{\text{active}}$.  For example, with $K=4$, $\Theta_{\max}=8\times10^6$ parameters and $L=128$, the active parameters per token are roughly $4.1$ billion.  This bounded compute property allows the total parameter universe to scale toward quadrillions without requiring quadrillion‑scale computation.

### 3.5 Rust Implementation and Type Mapping

The Rust implementation leverages the language’s memory‑safety to manage large parameter banks safely.  Core concepts are mapped into types:

* **TierId** – a lightweight identifier for tiers.  Predefined constants (e.g., `TIER_1B`, `TIER_10B`) represent standard capacities.
* **TierConfig** – metadata for each tier: parameter budget, number of groups, experts per group, routing hyperparameters and placement hints.
* **TierSet** – a collection of active tiers (with an option to include all tiers up to a certain capacity).  TierSet is passed to routing functions.
* **RoutingConfig** – parameters such as $k_{\text{tier}}, k_g, k_e$ and the deterministic seed.
* **ExpertKey** and **Route** – keys to index experts and routing decisions.  A `Route` records the tier, group, expert, weight and priority selected for a token.
* **Router** trait – exposes a `route()` method that takes a tensor and a TierSet and returns selected routes.  Implementations may differ (e.g., CPU vs. GPU kernels), but the interface remains stable.
* **ExpertStore** trait – fetches experts on demand given an `ExpertKey` and manages their placement across memory tiers.
* **MoELayer** – encapsulates a router and an expert store.  Its `forward()` method handles token packing, expert execution and result aggregation.
* **ModelConfig** – describes the overall model (vocabulary size, hidden dimension, number of layers, number of heads, TierConfig list).  The full type mapping is provided in `lite_llm_types.rs` in the repository.

The use of traits and generics allows backend‑agnostic implementations: the same MoE logic can be executed on CPUs, GPUs or specialized accelerators by providing appropriate `Device` and `ExpertStore` implementations.  Rust’s ownership model and compile‑time checks help prevent memory corruption and data races when transferring large tensors and parameter shards between hosts and devices.

## 4 Distributed System and Parallelism

Large models require parallelism across multiple devices and nodes.  Lite LLM employs **four parallelism dimensions**:

* **Data parallel (DP).** Different minibatch shards are processed on separate data‑parallel ranks.  DP is combined with gradient accumulation and sharded optimizers.
* **Tensor parallel (TP).** Matrix multiplications in attention and expert MLPs are split across devices along the hidden dimension.  TP reduces memory per device and increases arithmetic intensity.
* **Pipeline parallel (PP).** Layers are divided across pipeline stages; activations flow forward and backward through stages.  PP helps distribute deep models across limited device memory.
* **Expert parallel (EP).** Experts are distributed across expert‑parallel ranks.  After routing decisions, tokens are packed and all‑to‑all communication sends token data to the devices holding the selected experts.  Each device runs its experts on the incoming token subset and returns the outputs.  The communication complexity per rank is $O\bigl(\frac{N L K d}{R_{\text{EP}}}\bigr)$ for $N$ tokens, $L$ layers, hidden dimension $d$ and $R_{\text{EP}}$ expert‑parallel ranks.  Balanced routing and load‑balancing losses ensure that assignments per rank stay close to $A/R_{\text{EP}}$ where $A=N\cdot L\cdot K$ is the number of assignments.

These parallelism modes compose naturally.  For example, a 1 trillion–parameter model might use DP=256, TP=8, PP=2 and EP=32; a quadrillion‑parameter model would use additional EP and tier partitioning.  Deterministic routing ensures that all ranks agree on routing decisions before token packing.  A seeded hash is used as the tie‑breaker to avoid nondeterministic all‑to‑all mismatches.

## 5 Training

### 5.1 Curriculum and Expansion Strategy

Training a model with trillions of parameters must be phased.  Lite LLM employs a **curriculum**:

1. **Phase 0 – Base Training:** Train the transformer backbone and the smallest tier (e.g., *tier_1b*) until convergence.  Only the base and small experts are active.
2. **Phase 1 – Add Next Tier:** Introduce a new tier (e.g., *tier_10b*).  Freeze the base transformer and existing tiers, initialize the new experts and their routing heads, and train them while keeping the old parameters fixed.  Auxiliary load‑balancing losses (see below) are enabled to encourage routing diversity.
3. **Phase 2 – Joint Tuning:** Unfreeze all tiers and jointly fine‑tune the model with a small learning rate, optionally regularizing the new tiers to avoid catastrophic forgetting.
4. **Repeat:** Additional tiers (100 B, 1 T, etc.) are added using the same procedure.  Because of the deterministic TierSet design, checkpoints remain backward compatible—old tiers continue to function even if new tiers are added later.

### 5.2 Loss Functions

The primary loss is the task‑specific cross‑entropy or language modelling objective.  To prevent **expert collapse** (where only a few experts receive most assignments), Lite LLM adds hierarchical load‑balancing losses.  For a batch of tokens, let $u_{t}$ be the empirical marginal probability of selecting tier $t$, $u_{t,g}$ the marginal for group $g$ in tier $t$, and $u_{t,g,e}$ the marginal for expert $e$ in $(t,g)$.  The load‑balancing loss penalizes deviations from the uniform distribution:

\[
\mathcal{L}_{\text{lb}} = \alpha_t\, \mathrm{KL}(u_\mathcal{T} \Vert \mathrm{Unif}(\mathcal{T}))
 + \alpha_g\, \sum_{t\in\mathcal{T}} \mathrm{KL}(u_{t,\cdot} \Vert \mathrm{Unif}(G_t))
 + \alpha_e\, \sum_{t\in\mathcal{T}} \sum_{g=1}^{G_t} \mathrm{KL}(u_{t,g,\cdot} \Vert \mathrm{Unif}(E_{t,g})).
\]

This penalty encourages the router to distribute tokens evenly across tiers, groups and experts.  A similar concept appears in the literature: auxiliary losses and noisy routing help mitigate situations where one expert dominates【520653549415222†L136-L149】.

### 5.3 Optimizers and Precision

Large models require memory‑efficient optimizers.  The architecture allows pluggable optimizers; common choices include AdamW, Adafactor or SGD with momentum.  We recommend using an optimizer with factorized second moments to reduce optimizer state.  For 1 trillion‑parameter models, mixed precision training (e.g., BF16 or FP16) is essential.  For quadrillion‑parameter models, int8 or int4 weight formats may be used for storage and inference, while training might still occur in BF16.

### 5.4 No‑Starvation Guarantee

During training, it is essential that each trainable expert receives gradient updates.  If $A_t$ is the number of token‑to‑tier assignments per step and $M_t$ the number of experts in tier $t$, the expected assignments per expert per step are $A_t/M_t$.  Provided this ratio stays above a small threshold $\rho$, the probability that an expert receives no assignments in a training window of $T$ steps decays exponentially, $\Pr(\text{no updates}) \le e^{-T\rho}$.  This guides the curriculum: new tiers should not contain more experts than can be exercised by the available throughput; otherwise the expert weights remain largely untrained.

## 6 Inference

At inference time, the user or system selects a TierSet based on latency, memory and cost requirements.  Running in “1 B mode” uses only *tier_1b*; running in “100 B mode” uses *tier_1b*, *tier_10b* and *tier_100b* cumulatively (or just *tier_100b* exclusively).  The following steps occur:

1. **Token Processing.** Tokens are embedded and passed through the transformer layers.  At each MoE layer, the router computes tier, group and expert scores.  The deterministic top‑$k$ operator selects $k_{\text{tier}}$, $k_g$ and $k_e$ as per configuration.
2. **Parameter Fetch.** For each selected expert, the runtime checks whether its parameters are in the hot cache (HBM).  If not, they are prefetched from warm (DRAM), cold (NVMe) or archival storage based on the placement hint.  The StrataServe blueprint for hierarchical parameter servers demonstrates that caching the working parameters in HBM while managing out‑of‑core weights in DRAM and SSD can deliver multi‑terabyte models efficiently【696435369415399†L100-L115】.
3. **Expert Execution.** Tokens are packed by expert and dispatched to the devices hosting those experts.  Each expert runs its MLP on its assigned token subset.  The results are routed back and combined (e.g., weighted sum).
4. **Output.** The process repeats for all layers.  The final hidden state goes through output projection to generate logits, and a softmax yields probabilities.  Sampling or greedy decoding then produces the next token.

TierSets can be selected manually by the user or automatically by solving a small knapsack problem: maximize utility subject to latency and memory budgets.  This enables dynamic adaptation of compute cost to user requirements.

## 7 Checkpointing and Scalability

### 7.1 Checkpoint Format

Checkpoints are stored as a manifest plus parameter shards:

* **Manifest.** Lists tiers, groups and experts; records tensor shapes and data types; includes hashes for integrity.  It also records the TierConfig parameters and routing hyperparameters.
* **Shards.** Parameter tensors are sharded by tensor parallel, pipeline parallel and expert parallel partitions.  Each shard is associated with a tier and may reside in separate files or object storage keys.  Checkpoints may compress shards or apply quantization.

### 7.2 Backward Compatibility

Adding a new tier does not invalidate existing checkpoints.  When loading a checkpoint, the runtime can choose to load only the tiers in the current TierSet.  Missing tiers are ignored; if new tiers are present, they can be loaded lazily.  This design enables incremental expansion toward quadrillion‑scale models without ever retraining from scratch.

### 7.3 Scaling to Quadrillion Parameters

Quadrillion‑parameter capacity is achieved by adding many large tiers (e.g., *tier_1t*, *tier_10t*, etc.) and keeping $K$ small.  At this scale, most tiers reside in cold storage and are only activated for specialized queries.  HSER ensures that only a few experts run per token, keeping per‑token compute manageable.  The hierarchical memory system maintains high throughput by caching the working set in GPU memory and prefetching additional experts as needed【696435369415399†L100-L115】.

## 8 Deployment Considerations

### 8.1 Hardware and Infrastructure

Running Lite LLM requires clusters of GPUs or specialized accelerators with high‑bandwidth interconnects (NVLink, NVSwitch) and access to large DRAM and NVMe pools.  The StrataServe paper illustrates that pooling GPU HBM with CPU DRAM and SSD into a coordinated hierarchy can train multi‑terabyte models efficiently【696435369415399†L100-L115】.  NVLink or similar interconnects enable peer‑to‑peer transfers without CPU involvement, reducing latency.  RDMA or specialized network fabrics handle inter‑node communication for data‑parallel and expert‑parallel collectives.

### 8.2 System Reliability and Safety

Memory safety is critical for managing extremely large parameter arrays.  By implementing Lite LLM in Rust, we reduce the risk of memory corruption and undefined behaviour.  The NSA/CISA report emphasises that MSLs such as Rust embed safety mechanisms directly into the language, preventing buffer overflows and other memory errors【443264331305965†L73-L78】 and shifting the safety burden away from developers【443264331305965†L32-L37】.  Rust’s borrow checker enforces strict ownership, preventing data races across threads and devices.  For resource management (HBM, DRAM, NVMe), Rust’s RAII (resource‑acquisition‑is‑initialization) pattern ensures deterministic deallocation.

### 8.3 Limitations

Lite LLM inherits some challenges from MoE systems.  Sparse routing can lead to irregular memory access patterns and complex cross‑device communication.  Deployment must account for these patterns to avoid under‑utilization and latency spikes.  Load‑balancing auxiliary losses help distribute traffic, but balancing at quadrillion scale remains an open research problem.  Training such models requires enormous compute and careful curriculum design to prevent expert collapse.  Finally, selecting appropriate TierSets automatically under strict latency constraints may require sophisticated scheduling algorithms.

## 9 Future Work

Potential directions include:

* **Adaptive Tiering.** Develop dynamic policies that choose which tiers to activate per query based on content or user profile.
* **Specialized Experts.** Train domain‑specific tiers (e.g., legal, medical, code) with separate routing heads, enabling the model to tap specialized knowledge when needed.
* **Prefetch Optimisation.** Use predictive models to prefetch experts from cold storage before they are needed, reducing latency.
* **Hardware Co‑Design.** Explore ASICs or advanced memory hierarchies (e.g., non‑volatile memory) optimized for tiered sparse models.

## 10 Conclusion

Lite LLM introduces a principled approach to scaling language models beyond trillions of parameters by decoupling capacity from compute.  Through a Tiered Parameter Architecture, deterministic Hierarchical Sparse Expert Routing and robust Rust implementation, it enables massive parameter universes while keeping per‑token cost bounded.  The design draws on lessons from modern MoE research【520653549415222†L58-L67】【520653549415222†L117-L124】 and hierarchical memory systems【696435369415399†L100-L115】.  Memory‑safe language adoption further strengthens reliability【443264331305965†L32-L37】.  While challenges remain, Lite LLM provides a scalable, extensible foundation for next‑generation language models reaching toward the quadrillion‑parameter regime.
