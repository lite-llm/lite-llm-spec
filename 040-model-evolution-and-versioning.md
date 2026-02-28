# SPEC‑040 — Model Evolution & Versioning

Large models evolve over time through improvements in architecture, tier composition and routing strategies.  Lite LLM must support upgrading models while maintaining backward compatibility, traceability and reproducibility.  This specification defines the versioning scheme and evolution procedures.

## 1 Version Identifiers

Each model instance carries a **model identifier** comprising:

* **Model name:** e.g., `lite-llm-base`.
* **Major version:** increments for breaking changes in architecture (e.g., adding new layer types or changing hidden sizes).
* **Minor version:** increments for backward‑compatible changes such as adding tiers or experts.
* **Patch version:** increments for bug fixes or performance tweaks that do not change model behaviour.

Example: `lite-llm-base-v2.1.3` denotes major version 2, minor version 1, patch 3.

## 2 Evolution Scenarios

1. **Tier expansion:** adding new tiers without altering existing layers (SPEC‑031).  This increments the minor version.
2. **Architecture change:** adding a new type of block or modifying hidden sizes.  This requires a major version bump because old checkpoints cannot be loaded directly into the new architecture.
3. **Hyperparameter adjustment:** changing learning rates or optimizer hyperparameters; recorded in metadata but does not change the model version.
4. **Bug fix:** fixing a determinism issue or minor code bug; increments the patch version.

## 3 Backward Compatibility

* **Parameter compatibility:** major version bumps may render old checkpoints incompatible.  The runtime may provide migration tools to convert old checkpoints to new formats if possible.
* **Tier compatibility:** minor version bumps should remain backward compatible.  Old clients may ignore unknown tiers when loading a newer checkpoint.
* **API compatibility:** RPC or API schemas should maintain backward‑compatible semantics across minor and patch versions.  Breaking changes require a major version increment.

## 4 Metadata and Documentation

Each version includes release notes describing changes, the rationale for version increments and any migration steps.  Metadata in the checkpoint manifest includes the model identifier and optional pointers to release notes.

## 5 Upgrade Procedure

1. **Plan:** assess whether the change is backward compatible.  Determine the new version number.
2. **Implement:** add new layers, tiers or changes.  Update metadata schema if needed.
3. **Migrate:** provide scripts or utilities to convert existing checkpoints if possible.
4. **Test:** verify determinism and performance on representative workloads.
5. **Release:** publish the new version with documentation.

Versioning ensures that models evolve in a controlled manner, enabling users to load appropriate checkpoints and maintain reproducible research practices.
