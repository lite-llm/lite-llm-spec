# SPEC‑029 — Checkpoint Manifest Specification

Checkpoints capture the state of the model, optimizer and runtime at a given point in time.  The **checkpoint manifest** is a structured document that lists all files required to restore a checkpoint and describes their metadata.  This specification defines the schema of the manifest and the semantics of its fields.

## 1 Motivation

Checkpoints must be portable, versioned and compatible across cluster sizes and tier configurations.  A manifest centralises information about parameter shards, tier assignments, version numbers and checksums.  It enables partial loading (e.g., selecting a subset of tiers) and incremental updates.

## 2 Manifest Structure

The manifest is stored in a human‑readable format such as JSON or YAML.  Its top‑level fields include:

* **model_id:** unique identifier for the model architecture and version.
* **epoch / step:** training progress at the time of checkpoint.
* **tiers:** list of defined tiers, each with `tier_id`, `name`, `size_budget` and `placement_policy`.
* **shards:** list of parameter shards.  Each entry contains:
  * **path:** relative path to the shard file.
  * **type:** `dense` or `expert`.
  * **tensor_name / expert_key:** identifies the parameter.
  * **shape:** shape of the shard.
  * **dtype:** data type (e.g., bf16, fp32).
  * **tier_hint:** initial tier placement.
  * **checksum:** for corruption detection.
  * **version:** integer incremented when the shard is updated.
* **optim_state:** location and description of optimizer state shards (SPEC‑037).
* **router_state:** router weights and seeds per layer.
* **metadata_version:** version of the manifest schema.

## 3 Semantics

1. **Tier independence:** a manifest may list shards for tiers that are not loaded in a given run.  The runtime can select a subset of tiers and ignore the rest.
2. **Versioning:** if a shard’s `version` in the manifest is higher than the version on disk, the disk copy is considered stale and must be reloaded.
3. **Checksum:** the runtime verifies each shard’s checksum upon loading.  Mismatches trigger re‑download or abort.
4. **Extensibility:** new fields may be added with higher `metadata_version` numbers.  Older runtimes should ignore unknown fields.

## 4 Loading Process

1. Read the manifest and parse its contents.
2. Validate the manifest version and structure.
3. For each shard entry that matches the selected TierSet and parallelism partition, locate the shard file and verify its checksum.
4. Load the shard into memory or schedule it for lazy loading (SPEC‑026).
5. Restore router and optimizer states.

## 5 Security

Manifests may include digital signatures to protect against tampering (SPEC‑052).  They should be stored in secure storage with access controls.

## 6 Example

```yaml
model_id: lite‑llm‑base
epoch: 5
step: 10_000
tiers:
  - tier_id: 1
    name: hot
    size_budget: 40GB
    placement_policy: prioritized
  - tier_id: 2
    name: warm
    size_budget: 200GB
    placement_policy: lru
shards:
  - path: dense/layer_0/attention_q_0_0_of_2.bin
    type: dense
    tensor_name: attention_q
    shape: [512, 1024]
    dtype: bf16
    tier_hint: hot
    checksum: "abc123"
    version: 1
  - path: experts/tier_2/group_0/expert_1.bin
    type: expert
    expert_key: [2,0,1]
    shape: [4096, 512]
    dtype: bf16
    tier_hint: warm
    checksum: "def456"
    version: 0
optim_state:
  path: optim/state_0.bin
router_state:
  path: router/state_0.bin
metadata_version: 1
```

This example illustrates a simplified manifest for two tiers.  Real manifests may contain thousands of shard entries.

By formalising the checkpoint manifest, Lite LLM can reliably save and restore model state across distributed training runs and evolving storage topologies.
