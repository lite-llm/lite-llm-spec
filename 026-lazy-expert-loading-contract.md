# SPEC‑026 — Lazy Expert Loading Contract

In Lite LLM, trillions of parameters are stored across multiple tiers yet only a small subset are needed for any given input【815052634867799†L135-L146】.  Loading all experts into memory would be wasteful.  The **lazy expert loading contract** ensures that experts are loaded on demand and unloaded when no longer needed, while maintaining determinism and safety.

## 1 Goals

* **Demand‑driven:** load an expert only when the router selects it.
* **Deterministic:** ensure that the same experts are loaded in the same order given the same inputs, seeds and TierSet.
* **Safe concurrency:** prevent races between loading, evicting and modifying experts.
* **Auditability:** record loading operations for debugging and compliance.

## 2 Loading Sequence

1. **Selection:** the router selects a set of experts (SPEC‑005) for a token and returns their identifiers.
2. **Availability check:** for each expert, the cache manager (SPEC‑022) checks whether it is present in the hot cache or staged in the warm tier.
3. **Load request:** if not present, the runtime issues an asynchronous load request to the warm or cold tier.  The request includes the expert key, tier and expected size.
4. **Transfer:** the expert is streamed into a staging buffer (SPEC‑023, SPEC‑024).  Version and checksum are verified.
5. **Activation:** once loaded, the expert pointer is inserted into the cache and execution may proceed.
6. **Reference counting:** the runtime increments the reference count on the expert to prevent eviction during use.  After the kernel finishes, the reference count is decremented.

## 3 Determinism Considerations

* **Ordering:** experts must be loaded in a deterministic order to avoid out‑of‑order side effects (e.g., if multiple experts load concurrently, the runtime must define a total ordering or use globally synchronized tags).
* **Seed dependence:** if the router uses stochastic gating (e.g., noisy top‑k), the seeds used to generate noise must be recorded so that lazy loading decisions are reproducible.
* **Version locking:** training updates that modify an expert should lock the expert during update to prevent a concurrent eviction or replacement.

## 4 Unload and Cleanup

When the cache eviction policy (SPEC‑027) chooses to remove an expert:

1. Ensure the expert’s reference count is zero (no kernels are using it).
2. If the expert has been modified, write back to the lower tier (write‑through ensures modifications have already been propagated).
3. Remove the expert from the cache and free GPU memory.
4. Decrement the warm tier’s occupancy; if the warm tier is full, demote to cold.

## 5 Telemetry and Logging

Lazy loading operations are recorded in the audit log (SPEC‑056) with timestamps, expert identifiers, tier sources and outcomes (success/failure).  These logs help diagnose bottlenecks and are essential for reproducing behaviour.

By formalising the lazy expert loading contract, Lite LLM enables efficient, deterministic operation across huge parameter spaces while retaining the ability to debug and audit every expert activation.
