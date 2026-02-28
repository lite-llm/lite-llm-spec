# SPEC‑028 — Parameter Sharding Format

To scale models to trillions of parameters, Lite LLM shards parameter tensors across many files and devices.  The **parameter sharding format** defines how these shards are organised, named and stored on disk so that they can be loaded in parallel and incrementally.

## 1 Design Goals

* **Scalability:** support parameter sets whose total size exceeds the capacity of any single device or file system.
* **Parallelism:** allow multiple shards to be loaded concurrently across tensor and expert parallel ranks.
* **Tier awareness:** record tier placement and routing metadata with each shard.
* **Data integrity:** include checksums and versioning to detect corruption.

## 2 Sharding Scheme

### 2.1 Dense Parameters

Dense parameters (embeddings, attention weights) are partitioned by row or column depending on the tensor parallel strategy (SPEC‑012).  Each shard includes:

* **Tensor name and layer index.**
* **Shard index and total number of shards.**
* **Shape of the local shard.**
* **Data type and precision.**

### 2.2 Expert Parameters

Experts are stored as separate files or objects.  Each expert file contains:

* **Expert key (tier, group, expert id).**
* **Parameter tensors for the expert’s layers (e.g., weight matrices for W1 and W2).**
* **Placement hint:** initial tier placement (hot, warm, cold, archive).
* **Checksum:** used during lazy loading to verify correctness.

Experts may be grouped into larger container files for efficient storage; offsets and lengths are recorded in the manifest.

## 3 File Naming Conventions

Shards are named using a deterministic scheme:

```
{model_name}/
  dense/
    layer_{l}/{tensor}_{rank}_{of}_{n}.bin
  experts/
    tier_{t}/group_{g}/expert_{e}.bin
```

This hierarchy makes it easy to locate shards given a tensor index or expert key.  The deterministic naming ensures cross‑node reproducibility.

## 4 Metadata Indexing

A metadata file accompanies each collection of shards.  It lists all shards with their file paths, sizes, offsets, placement hints, checksums and version numbers.  The metadata file itself has a checksum and is referenced in the checkpoint manifest (SPEC‑029).

## 5 Integration with Storage Tiers

Shards may live across multiple tiers.  The index records the storage location (e.g., hot, warm, cold, archive).  During loading, the runtime consults this index to locate and fetch each shard.  The format supports relocating shards between tiers by updating placement hints in the manifest.

By formalising parameter sharding, Lite LLM ensures that massive parameter sets can be split, stored, transferred and reassembled efficiently across distributed systems.
