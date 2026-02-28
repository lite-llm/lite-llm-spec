# Lite LLM Enterprise Runtime Specification 008: Error Model & Failure Domains

## Purpose

This specification categorizes runtime errors and defines how the system responds to them.  It distinguishes recoverable and fatal errors, outlines routing mismatch detection, explains expert starvation handling and describes cross‑node failure behaviour.

## Error Categories

Lite LLM errors fall into four categories:

1. **Configuration Errors:** Missing or invalid configuration (e.g., unknown Tier ID, mismatched group counts).  Detected at startup; the runtime aborts with a descriptive message.
2. **Resource Errors:** Insufficient memory, exhausted file descriptors, or invalid device states.  These may be recoverable via fallback strategies (e.g., demote a tier to make room) or fatal if essential resources cannot be allocated.
3. **Routing Errors:** Mismatches in routing assignments across nodes or invalid expert IDs.  Such errors could cause incorrect all‑to‑all communication and must be detected and handled promptly.
4. **Runtime Exceptions:** Unexpected panics in kernels, memory corruption or arithmetic exceptions (e.g., NaNs).  The error model requires deterministic handling: replicate the error across replicas or abort and restart.

## Recoverable vs Fatal Errors

* **Recoverable Errors:** Resource exhaustion, recoverable communication timeouts, cache misses that trigger lazy loading.  The runtime logs these errors, applies fallback policies (e.g., freeing unused tiers) and retries the operation.
* **Fatal Errors:** Corrupted checkpoints, version mismatches, routing consensus failures, or deterministic invariants violated.  The runtime aborts and enters the crash recovery procedure (SPEC 002).  Fatal errors may require manual intervention.

## Routing Mismatch Detection

Routing mismatches occur when different nodes compute divergent routing decisions for the same token.  To detect mismatches:

* **Checksum Exchange:** Each rank computes a checksum of its routing assignments for a batch and participates in an all‑reduce.  If checksums differ, an error is raised.
* **Debug Mode:** In debug builds, ranks may exchange full routing maps.  When a mismatch is detected, the runtime logs the offending layer and token index and aborts to preserve correctness.

## Expert Starvation Handling

Starvation arises when an expert receives zero assignments over long periods.  The curriculum and load balancing losses (SPEC 005, SPEC 031) mitigate this risk.  Additionally:

* **Monitoring:** The runtime tracks assignment counts per expert.  Low‑traffic experts may trigger alarms.
* **Reactivation:** Experts with persistent starvation can be re‑initialized or merged with more active experts.  Alternatively, the router may be re‑trained to distribute traffic better.

## Cross‑Node Failure Behaviour

When a node fails:

* **Detection:** Heartbeat mechanisms detect the failure and inform all other nodes.
* **Graceful Degradation:** Remaining nodes reduce parallelism degree (e.g., fewer expert parallel ranks).  Routing is recomputed to exclude the missing experts.
* **State Reconstruction:** A replacement node can reload the relevant tier shards and join the cluster.  Deterministic seeds ensure that routing assignments are reproducible after recovery.

## References

See distributed systems literature for fault models, such as Paxos/Raft for consensus.  The error model is inspired by practices in large‑scale parameter servers and GPU clusters.