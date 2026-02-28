# SPEC‑039 — Deterministic Training Replay Engine

Reproducing training runs exactly is essential for debugging, auditability and research reproducibility.  Lite LLM provides a **deterministic training replay engine** that re‑executes a training run step‑by‑step and reproduces parameter updates, routing decisions and results exactly.

## 1 Requirements

* **Bitwise equivalence:** parameter values, gradients and activations must match across the original and replay runs.
* **Consistent control flow:** the sequence of operations, including micro‑batch ordering and tier expansion, must be identical.
* **Identical random seeds:** any randomness (e.g., dropout, noisy routing) must use the same seeds.
* **Snapshot restoration:** start from a specific checkpoint and replay the same data in the same order.

## 2 Replay Inputs

To replay a run, the following artefacts are required:

1. **Checkpoint manifest and shards** (SPEC‑029) representing the starting state.
2. **Training log:** a sequence of events, including data shard ordering, random seeds for each step, learning rate schedule and tier expansion schedule.
3. **Configuration:** hyperparameters, TierSet and routing hyperparameters.

## 3 Replay Procedure

1. **Restore state:** load the checkpoint as described in SPEC‑030.
2. **Seed initialisation:** set global seeds for RNGs (Python, CUDA, Rust) using the values recorded in the training log.
3. **Data order:** replay the same sequence of mini‑batches in the same order.  Data must be stored or accessible via deterministic indexing.
4. **Execute steps:** for each step, perform forward and backward passes, apply gradient accumulation, execute deterministic collectives (SPEC‑018), update parameters using the same optimizer and apply any tier expansion events according to the schedule.
5. **Verify:** after each step or at chosen checkpoints, compare parameter hashes or checksums against the expected values.  Any divergence indicates a determinism bug.

## 4 Challenges and Solutions

* **Non‑deterministic hardware:** certain GPU operations (e.g., atomic add) may be nondeterministic.  The runtime must avoid or replace these operations with deterministic kernels.
* **Different world sizes:** replaying with a different number of data parallel workers can introduce nondeterminism due to different reduction orders.  The replay engine may restrict to the original world size or adjust seeds and ordering accordingly.
* **External sources of nondeterminism:** environment variables, hardware interrupts and library versions can cause divergence.  Capture these during the original run and replicate them.

## 5 Applications

* **Debugging:** diagnose gradient explosions, training instabilities or unusual routing behaviour by reproducing the exact conditions.
* **Auditing:** demonstrate compliance with regulatory requirements by proving that training followed documented procedures.
* **Research:** test hypotheses about training dynamics by replaying runs and modifying specific components while keeping everything else constant.

By providing a deterministic training replay engine, Lite LLM empowers engineers and researchers to understand and reproduce the complex dynamics of trillion‑parameter training runs.
