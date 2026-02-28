# SPEC‑020 — Fault‑Tolerant Distributed Execution

Distributed training at scale inevitably encounters node failures, network partitions and transient hardware errors.  Lite LLM must detect, isolate and recover from such faults without corrupting model state or sacrificing determinism.  This specification describes failure domains, error handling strategies and recovery procedures for the distributed runtime.

## 1 Fault Model

Lite LLM considers the following fault types:

1. **Process failure:** a worker process crashes or is forcibly terminated.
2. **Node failure:** a machine (host) goes offline or loses network connectivity.
3. **Network partition:** messages between subsets of nodes are dropped or delayed indefinitely.
4. **Device error:** GPU or accelerator failure, memory parity error or fatal driver error.
5. **Storage error:** read/write failure on local or remote tiers.

## 2 Failure Detection

* **Heartbeat monitoring:** each rank periodically sends heartbeats to a coordinator.  Missing heartbeats beyond a threshold triggers suspicion of failure.
* **Timeouts:** deterministic collective operations include timeouts; if a rank fails to participate, the coordinator marks it as failed.
* **Consistency checks:** checksum mismatches in routing consensus (SPEC‑016) indicate a transient memory or network error and trigger a retry.

## 3 Error Handling Strategies

1. **Transient errors:** temporary network delays or storage hiccups.  The runtime retries communication with exponential back‑off.  Deterministic collectives require coordinated retries to avoid duplicate reductions.
2. **Recoverable errors:** a GPU memory parity error may allow the process to continue after clearing the affected buffer.  The runtime marks the relevant tensor as corrupted and reloads it from checkpoint.
3. **Process failure:** if a worker crashes, remaining ranks pause at the next synchronisation point.  The coordinator either restarts the failed process and resynchronises it (elastic recovery) or aborts the entire step and restarts from the latest checkpoint.
4. **Node failure:** cluster orchestrators (e.g., Kubernetes) replace failed nodes.  The runtime reloads shards of the model onto new nodes and re‑initialises the transport.  Tier placement (SPEC‑021) ensures that cold tiers are resilient to individual node failures.
5. **Irrecoverable errors:** multiple simultaneous failures or corrupted checkpoints may force a full abort.  In this case, the run is terminated and a previous checkpoint is restored.

## 4 Checkpoint‑Based Recovery

Lite LLM uses distributed checkpoints (SPEC‑038) to recover from catastrophic failures.  Checkpoints include the model weights, optimizer states, tier metadata and routing configuration.  On recovery:

1. The cluster is re‑initialised according to the checkpoint manifest (SPEC‑029).
2. Each rank restores its share of the weights and optimizer states from the checkpoint.
3. The transport layer re‑establishes connections.
4. Routing seeds are reset to their values at the checkpoint to maintain determinism.

## 5 Elastic Training

Elastic training allows the number of workers to vary during a run.  When nodes join or leave due to scaling or failures, the runtime redistributes data and parameter shards across the new world size.  Elasticity interacts with determinism because reorderings of collectives may change the reduction ordering.  To preserve determinism, Lite LLM only allows elasticity at well‑defined boundaries (e.g., at the beginning of an epoch) and re‑seeds the routing engine.

## 6 Testing and Verification

* **Fault injection:** automated tests deliberately kill processes, drop network packets or corrupt memory to verify recovery logic.
* **Consistency checks:** after recovery, verify that model parameters match the checkpoint, ensuring no partial updates were applied.
* **Performance:** measure the overhead of fault handling to ensure minimal impact on steady‑state throughput.

By defining clear failure domains and recovery procedures, Lite LLM ensures that distributed training can withstand faults while preserving determinism and model correctness.
