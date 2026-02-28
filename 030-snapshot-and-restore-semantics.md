# SPEC‑030 — Snapshot & Restore Semantics

Snapshots capture a consistent view of the model and optimizer state at a moment in time.  Restore semantics define how snapshots are applied to resume training or inference after interruption.  This specification codifies snapshot triggers, consistency models and restoration procedures.

## 1 Snapshot Triggers

Snapshots may be taken:

* **Periodically:** at fixed intervals (e.g., every N steps or epochs).
* **Before major events:** prior to tier expansion (SPEC‑031), hyperparameter changes or data curriculum transitions.
* **On demand:** when explicitly requested for debugging or evaluation.
* **Before risk of failure:** ahead of scheduled maintenance or anticipated outages.

## 2 Consistency

A snapshot must represent a globally consistent state.  This is achieved by:

1. **Barrier:** all ranks reach a synchronization point, ensuring that all pending gradients have been applied.
2. **Freeze updates:** parameter updates and routing changes are paused during snapshot writing.
3. **Serialize state:** each rank writes its parameter shards, optimizer state shards and routing state concurrently to the storage backend.
4. **Write manifest:** generate the checkpoint manifest (SPEC‑029) listing all shards and their metadata.
5. **Unfreeze:** resume training or inference once snapshot writing completes.

## 3 Atomicity

Snapshots must be applied atomically.  Partial snapshots (e.g., due to failure during writing) must not be considered valid.  To enforce atomicity:

* **Two‑phase commit:** write snapshot files with temporary names, then commit them by renaming once all writes succeed.
* **Version number:** increment the checkpoint version only after successful commit.  The runtime loads the highest complete version.
* **Lock file:** maintain a lock or lease that prevents concurrent snapshot writers.

## 4 Restoration Process

When restoring from a snapshot:

1. **Select snapshot:** choose the desired checkpoint manifest.  Optionally filter tiers to load a subset (TierSet).
2. **Validate:** verify manifest integrity and ensure that all referenced files exist and checksums match.
3. **Allocate resources:** allocate devices, memory and network connections based on the target world size and parallel configuration.
4. **Load shards:** for each shard in the manifest, load or schedule it for lazy loading (SPEC‑026).  Restore router state and optimizer state.
5. **Re‑seed:** reset routing seeds to the values recorded in the manifest to maintain determinism.
6. **Resume execution:** continue training or begin inference from the restored state.

## 5 Backward Compatibility

Snapshots created under older versions of the runtime must remain loadable.  When fields are added to the manifest, the loader should ignore unknown fields or map them to default values.  If older snapshots lack fields required by the new runtime, an upgrade script must fill in defaults or transform the snapshot.

## 6 Security

Snapshot files may be signed and encrypted (SPEC‑052, SPEC‑053).  Restoration requires appropriate decryption keys and integrity verification.  Audit logs record restoration events (SPEC‑056) for compliance.

Through robust snapshot semantics, Lite LLM ensures recoverability and reproducibility across training sessions and infrastructure failures.
