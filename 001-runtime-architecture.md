# Lite LLM Enterprise Runtime Specification 001: Runtime Architecture Overview

## Purpose

This specification provides a high‑level overview of the Lite LLM runtime architecture.  It establishes system goals, defines bounded compute guarantees, introduces the Tiered Parameter Architecture (TPA) and Hierarchical Sparse Expert Routing (HSER), and outlines typical deployment topologies for enterprise environments.

## System Goals

* **Scalability:** Enable language models with total parameter counts ranging from billions to quadrillions without proportional increases in per‑token compute or latency.  The architecture decouples **capacity** from **active computation** via sparse activation.
* **Determinism:** Ensure that routing and execution are reproducible across runs and across distributed processes.  Deterministic behaviour is critical for debugging, auditing and safety compliance.
* **Modularity:** Compose parallelism strategies (data, tensor, pipeline, expert) with a hierarchical storage system to accommodate diverse hardware configurations.  Each component—router, expert store, prefetcher—has a defined interface that permits independent optimization.
* **Security:** Implement memory‑safe operation through Rust and enforce strict access control for sensitive model tiers.

## Bounded Compute Guarantees

Lite LLM guarantees that the number of active parameters per token is bounded by

\[
P_{\text{active}} \le L\bigl(C_{\text{dense}} + K\,\Theta_{\max}\bigr),
\]

where $L$ is the number of transformer layers, $C_{\text{dense}}$ is the cost of dense attention and normalization, $K$ is the product of top‑$k$ selections across the routing hierarchy, and $\Theta_{\max}$ is the largest expert size.  Regardless of total capacity, only a fixed number of experts are executed per token.  This bound enables predictable latency and cost at quadrillion‑parameter scale.

## Tiered Parameter Architecture (TPA)

TPA organizes parameters into discrete tiers (e.g., *tier_1b*, *tier_10b*, *tier_100b*, *tier_1t*, etc.).  Each tier has its own:

* **Parameter budget** – maximum total parameters allocated to experts in the tier.
* **Placement policy** – whether parameters reside in GPU HBM, CPU DRAM, NVMe, or remote object storage.
* **Routing heads** – scoring functions that map token features to tier probabilities.

Tiers enable dynamic capacity selection.  A *TierSet* defines which tiers are active for a given request, permitting trade‑offs between accuracy, latency and resource consumption.

## Hierarchical Sparse Expert Routing (HSER)

HSER is a three‑level deterministic gating mechanism:

1. **Tier Router:** Scores each tier in the tier universe and selects $k_{\text{tier}}$ tiers based on a seeded top‑$k$ operation.
2. **Group Router:** Within each selected tier, scores each group and selects $k_g$ groups.
3. **Expert Router:** Within each selected group, scores each expert and selects $k_e$ experts.

The cross product of these selections yields at most $k_{\text{tier}} \times k_g \times k_e$ active experts per token.  The deterministic seed ensures reproducibility.

## Deployment Topologies

Lite LLM can be deployed in various topologies depending on model size and target latency:

* **Single‑node (development):** All tiers reside on a single server.  Suitable for experimentation or small models.
* **Multi‑GPU node:** Experts are distributed across multiple GPUs with NVLink or NVSwitch interconnects.  Tiers may span HBM and DRAM.
* **Cluster:** Data parallel, tensor parallel, pipeline parallel and expert parallel nodes communicate via RDMA or high‑speed Ethernet.  Parameter storage is hierarchical, and object storage holds cold tiers for on‑demand loading.
* **Hybrid cloud/edge:** Edge nodes host hot tiers and perform inference close to users; cold tiers reside in the cloud.  Deterministic routing ensures that token paths are consistent across deployments.

## References

See **References.md** for foundational literature on sparse Mixture‑of‑Experts, deterministic routing and hierarchical memory systems.