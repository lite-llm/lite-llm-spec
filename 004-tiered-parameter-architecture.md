# Lite LLM Enterprise Runtime Specification 004: Tiered Parameter Architecture (TPA)

## Purpose

This specification defines the Tiered Parameter Architecture (TPA), which partitions model parameters into discrete tiers.  TPA provides a structured way to scale capacity, manage memory placement, and maintain backward compatibility as new tiers are introduced.

## Tier Metadata Schema

Each tier is defined by a metadata record containing:

* **Tier ID:** A unique integer or string identifier (e.g., `tier_1b`, `tier_10b`).
* **Parameter Budget:** The maximum total number of parameters allocated to experts in the tier.
* **Groups per Tier:** $G_t$ groups.  Each group contains experts that may share structural characteristics (e.g., hidden dimension).
* **Experts per Group:** $E_{t,g}$ experts within group $g$ of tier $t$.
* **Placement Policy:** A hint indicating the default storage medium (HBM, DRAM, NVMe, object store) used for the tier’s parameters.
* **Routing Hyperparameters:** Values for $k_{\text{tier}}, k_g, k_e$ when this tier is active.
* **Compatibility Flags:** Additional fields capturing version compatibility, data types and optional quantization schemes.

The metadata schema is stored in the checkpoint manifest and can be serialized/deserialized independently of the parameters themselves.

## Parameter Budgeting Rules

Budgets ensure that tiers do not accidentally exceed their intended scale:

* The total parameter count of all experts in a tier must not exceed the tier’s declared budget.  Runtime can enforce this at load time by summing the sizes of expert parameter tensors.
* Budgets are advisory for planning but are not strictly enforced at runtime; a tier may contain fewer parameters than its budget.
* When defining new tiers, budgets should align with hardware capabilities and caching strategies (e.g., a 10 B tier fits in DRAM but not HBM).

## Tier Compatibility Rules

TPA requires that tiers be backward compatible across model versions.  Compatibility rules include:

* **Freezing Base Layers:** Dense parameters (embeddings, attention, norms) remain unchanged across tiers.  New tiers may introduce additional router heads but must not modify existing dense weights.
* **Non‑Intrusive Additions:** New tiers may define new groups and experts, but they must not alter existing group/expert structures.  Routing functions must handle missing tiers gracefully.
* **Manifest Versioning:** A manifest includes a version number.  The runtime checks version compatibility and either loads the checkpoint or rejects it with a descriptive error.

## Tier Expansion Contract

To add a new tier $t_{\text{new}}$:

1. Choose a unique `Tier ID` and assign a parameter budget and placement policy.
2. Define the number of groups and experts per group.  The sum of parameters of these experts must not exceed the budget.
3. Add routing heads for the new tier at each MoE layer.  These heads can be trained separately while keeping existing weights frozen.
4. Update the manifest with the new tier metadata and record the creation timestamp.  Existing tier definitions remain unchanged.
5. Train the new tier via a curriculum: freeze previous tiers, train the new tier with load balancing losses, then optionally fine‑tune jointly.

## Backwards Compatibility Guarantees

The contract above ensures that old checkpoints continue to function when new tiers are introduced:

* Older runtimes can ignore unknown tiers in the manifest and still load all known tiers.
* New runtimes can load old manifests by assigning default placement policies to missing metadata fields.
* Routing functions mask out tiers not present in the active TierSet, so adding tiers does not disrupt existing model behaviour.

## References

TPA draws inspiration from mixture‑of‑experts scaling literature and hierarchical memory management.  See **References.md** for additional sources.