# Lite LLM Enterprise Runtime Specification 016: Routing Consensus Protocol

## Purpose

In a distributed environment, each rank must agree on which experts a token should visit.  This specification describes the routing consensus protocol that ensures consistent routing decisions across all ranks.

## Overview

Routing involves computing scores on each rank and selecting experts via the deterministic top‑$k$ algorithm (SPEC 003).  Because hidden states may be partitioned across tensor parallel ranks, consensus is required to ensure that all ranks use the same routing decisions.

## Steps in the Consensus Protocol

1. **Hidden State Synchronization:** Before routing, tensor parallel ranks use an all‑reduce or all‑gather to reconstruct the full hidden state for each token.  Deterministic collective operations (SPEC 018) ensure identical values.
2. **Local Score Computation:** Each rank computes scores $z_t, z_{t,g}, z_{t,g,e}$ using its copy of router weights.  Since weights are replicated or sharded deterministically, scores are identical across ranks.
3. **Stable Top‑$k$ Selection:** Each rank independently applies the stable top‑$k$ algorithm with the same seed to produce tier, group and expert selections.
4. **Checksum Exchange:** Optionally, ranks exchange a checksum of selected expert IDs for each token.  Mismatches trigger an error (SPEC 008) and abort execution.

The protocol leverages deterministic seeding and shared weights to avoid expensive coordination.  Unlike consensus algorithms such as Paxos, no explicit leader election or majority votes are needed for every decision; the deterministic algorithm suffices.

## Failure Handling

If a rank experiences a failure during routing consensus:

* **Timeout:** Other ranks detect a timeout and abort the current batch.  The failed rank is removed from the group, and routing decisions for future batches are recomputed without it.
* **Replay:** Batches may be replayed on the remaining ranks to ensure that output consistency is maintained.

## Determinism Assurance

The consensus protocol is only valid if the underlying operations are deterministic:

* Hidden state synchronization must use deterministic collectives.
* Router weights and seeds must be identical on all ranks.
* The stable top‑$k$ algorithm must produce identical results regardless of hardware differences.

## References

Consensus protocols are well studied in distributed systems.  Lite LLM’s routing consensus is simplified due to deterministic algorithms and does not rely on Paxos or Raft for each routing decision.  For more details, see **References.md**.