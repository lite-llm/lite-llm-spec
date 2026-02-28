# Lite LLM Enterprise Runtime Specification 002: Process Model & Execution Lifecycle

## Purpose

This specification defines the lifecycle of the Lite LLM runtime from initialization through execution and shutdown.  It formalizes the boot sequence, model loading phases, TierSet activation, and recovery procedures for unexpected failures.

## Boot Sequence

1. **Configuration Loading:** Read the runtime configuration, including available tiers, TierSet policies, routing hyperparameters and device assignments.
2. **Resource Allocation:** Initialize the asynchronous runtime.  Allocate device buffers for tokens, activations and the hot cache.  Reserve staging space for warm and cold tiers in DRAM and NVMe.
3. **Routing Seed Initialization:** Generate or load a deterministic seed for routing.  The seed must be consistent across all nodes for reproducibility.
4. **Logging Subsystem:** Start deterministic logging and telemetry collection.  All subsequent events are recorded for auditability.
5. **Model Loading:** See the next section for details.

## Model Loading Phases

1. **Manifest Parsing:** Read the checkpoint manifest.  Verify version compatibility and integrity checksums.  Resolve the TierSet to load (e.g., {tier_1b, tier_10b}).
2. **Base Parameter Loading:** Load shared parameters (embeddings, attention weights, norm weights) from the manifest.  Distribute shards across tensor and pipeline parallel ranks according to the specification.
3. **Expert Registration:** For each active tier, register expert groups and create `ExpertKey` entries.  Load experts into the hot cache if their placement policy is HBM, or into staging buffers if DRAM/NVMe.
4. **Router Parameter Loading:** Load tier, group and expert router weights.  Initialize router state machines.
5. **Optimizer State Loading (if training):** Load sharded optimizer states.  For inference, this step is skipped.

## TierSet Activation

After loading, the runtime activates a TierSet for the current session.  Activation entails:

* Registering which tiers are eligible for selection by the tier router.  Tiers not in the TierSet are masked out.
* Adjusting the placement policy for each tier: tiers in the TierSet may be promoted from cold to warm storage.
* Initializing auxiliary scheduling structures to enforce latency budgets.

TierSet activation may occur at startup or be changed dynamically via an API call.  Changing the TierSet triggers a reconfiguration where new tiers are loaded and obsolete tiers are demoted.

## Shutdown & Crash Recovery

### Graceful Shutdown

1. **Quiesce New Requests:** Stop accepting new requests; allow in‑flight requests to complete.
2. **Flush Telemetry:** Drain telemetry buffers and write logs to durable storage.
3. **Persist Hot Cache:** If caching modifications must be preserved (e.g., after tier fine‑tuning), write out modified experts.
4. **Release Resources:** Deallocate device buffers and close file handles.

### Crash Recovery

* **Crash Detection:** The runtime monitors heartbeats between nodes.  On failure, a supervisor process records the failure time and captures logs.
* **State Restoration:** On restart, parse the manifest and reload the same TierSet.  Deterministic seeds and logs ensure that the state before the crash can be reconstructed.
* **Replica Resynchronization:** Data‑parallel replicas recover their optimizer states (if training).  Expert parallel ranks reload the same expert shards.  Consistency is re‑established via deterministic collective operations.

## References

Refer to **References.md** for related work on system process models and distributed deep learning lifecycle management.