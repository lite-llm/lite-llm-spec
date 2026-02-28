# SPEC‑038 — Distributed Checkpointing

With trillions of parameters, writing checkpoints from a single process would be prohibitively slow.  Lite LLM implements **distributed checkpointing**, where each rank writes its own shards concurrently.  This specification outlines the protocol for efficient and consistent checkpoint creation.

## 1 Design Objectives

1. **Throughput:** maximise write bandwidth by parallelising file writes across ranks.
2. **Consistency:** ensure that all shards belong to the same model state (see SPEC‑030 for snapshot semantics).
3. **Fault tolerance:** handle partial failures during checkpointing without corrupting existing checkpoints.
4. **Portability:** support writing to local disks, network file systems and object stores.

## 2 Checkpointing Procedure

1. **Synchronise:** all ranks reach a snapshot barrier (SPEC‑030) to freeze updates.
2. **Local write:** each rank writes its parameter shards, optimizer state shards and router state shards to files.  Names follow the deterministic format defined in SPEC‑028 and SPEC‑037.
3. **Manifest assembly:** a designated coordinator rank collects metadata about all shards (paths, sizes, checksums) from workers and assembles the checkpoint manifest (SPEC‑029).
4. **Commit:** after all files are successfully written, the manifest is written and atomically committed.  If any shard write fails, the checkpoint is aborted and intermediate files are cleaned up.
5. **Resume:** training resumes after the checkpoint is committed.

## 3 Parallel I/O Considerations

* **Bandwidth aggregation:** writing many small files can saturate network file systems; grouping shards or using larger aggregated writes can improve throughput.
* **Backpressure:** avoid overwhelming storage by limiting the number of concurrent writes per rank.
* **Compression:** optional compression reduces storage size but increases CPU overhead.  Compression algorithms must be deterministic.

## 4 Fault Tolerance

If a rank fails during checkpointing:

* The coordinator aborts the checkpoint.  Partially written files are removed.
* The system can either retry immediately or wait until the failed rank is recovered (SPEC‑020).
* Previous checkpoints remain untouched and can be used to recover.

## 5 Integration with Tier Placement

Checkpoint files may reside on different tiers.  For example, hot tier experts may be checkpointed to high‑throughput local disks, while cold tier experts may go directly to remote storage.  The manifest records the storage locations to enable correct restoration.

By distributing checkpoint writes across ranks and enforcing atomic commits, Lite LLM ensures efficient and reliable persistence of massive model states.
