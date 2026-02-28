# SPEC‑037 — Sharded Optimizer State Format

The state maintained by optimizers (e.g., momentum, second moments) can be comparable in size to the parameters themselves.  For trillion‑parameter models, storing optimizer state on a single device or tier is impossible.  This specification defines how optimizer state is sharded, stored and retrieved.

## 1 Sharding Strategy

* **Parameter alignment:** for each parameter shard (SPEC‑028), optimizer state is sharded along the same axis.  This simplifies lookup and loading.
* **State types:** store separate sharded files for each type of state (e.g., first moment `m`, second moment `v`, factorised rows and columns for Adafactor).  Each file includes metadata describing its alignment with the parameter shard.
* **Tier placement:** states for frequently updated (hot) parameters reside in GPU memory; states for warm and cold experts reside in DRAM or NVMe.  This mirrors the placement of the parameters.

## 2 File Format

An optimizer state shard file includes:

* **Parameter id and shard index.**
* **State name:** e.g., `m`, `v`, `rows`, `cols`.
* **Shape and dtype:** local shape and data type.
* **Version:** incremented when the state is updated.
* **Checksum:** for integrity checks.

Files are stored under `optim_state/` directories and are referenced by the checkpoint manifest (SPEC‑029).

## 3 Loading and Saving

When restoring a checkpoint:

1. Load parameter shards as usual.
2. For each parameter shard, load the corresponding optimizer state shards.  If a parameter does not have state (e.g., biases may not require second moment), skip loading.
3. If the state is sharded across devices, ensure that each rank loads the correct portion based on its tensor parallel rank.

When saving:

1. Write each state shard to its own file.  Use a temporary name and commit atomically (SPEC‑030).
2. Update the manifest with new checksums and version numbers.

## 4 Sparse Optimizer States

For experts that are rarely used, their optimizer states may be stored in compressed form or omitted entirely.  Upon first activation after a long hiatus, their states can be initialised lazily (e.g., zeros).  The optimizer abstraction (SPEC‑034) must handle missing state gracefully.

## 5 Tier Promotion and Demotion

When a parameter moves between tiers (SPEC‑021), its optimizer state must move with it.  The sharded format allows copying only the relevant shard rather than reserialising the entire state.

By defining a sharded state format, Lite LLM ensures that optimizer data scales with model size and can be efficiently stored, loaded and updated across tiers and parallel ranks.
