# Lite LLM Enterprise Runtime Specification 010: Runtime State Machine

## Purpose

This specification defines the state machine governing the Lite LLM runtime.  It enumerates model states, valid transitions and state invariants to ensure predictable behaviour through initialization, normal operation, expansion and recovery.

## States

1. **Init:** The runtime has started, but no model is loaded.  Configuration files are parsed, and seeds are initialized.  Only transitions to `Warm` are allowed.
2. **Warm:** The base model and initial tiers are loaded into memory.  Router weights are ready, but no requests are being processed.  The system may prefetch additional tiers based on configuration.
3. **Active:** The runtime is serving requests.  Tiers are selected via the TierSet, and routing decisions determine expert execution.  Training or inference tasks run concurrently.  Transitions to `Expanding` or `Frozen` may occur based on user commands.
4. **Expanding:** A new tier or new experts are being added.  Existing tiers remain active, but newly added tiers are loaded and initialized.  The system may temporarily reduce throughput during expansion.  After completion, the state returns to `Active`.
5. **Frozen:** The runtime stops processing new requests temporarily.  All existing tasks complete or are drained.  This state is used during maintenance operations, such as tier compaction or security updates.  After maintenance, the state transitions back to `Warm` or `Active`.
6. **Recovering:** The runtime is recovering from a crash or fatal error.  It reloads the previous TierSet, reconstructs optimizer and router states, and replays logs to restore determinism.  Once recovered, the state transitions to `Warm` or `Active`.

## Valid Transitions

* **Init → Warm:** After configuration and seed initialization, the model loads tiers and enters `Warm`.
* **Warm → Active:** When the runtime begins processing requests.
* **Active → Expanding:** When a new tier is being added via the curriculum protocol.
* **Expanding → Active:** After successful expansion and optional joint fine‑tuning.
* **Active → Frozen:** For maintenance or security updates.
* **Frozen → Warm/Active:** When maintenance is complete.
* **Any → Recovering:** On detection of a fatal error or node failure.
* **Recovering → Warm:** After state restoration.  Optionally transitions to `Active` if requests resume immediately.

## State Invariants

* **Deterministic Seeds:** The routing seed must remain constant across state transitions to preserve determinism.
* **Tier Integrity:** In `Active`, all active tiers have been successfully loaded.  During `Expanding`, new tiers are loaded without modifying existing tier metadata.
* **Logging Enabled:** Audit and telemetry logging remain active in all states except `Init` to ensure reproducibility and compliance.
* **Concurrency Locks:** Transitions to `Frozen` acquire locks to prevent new tasks from starting while allowing in‑flight tasks to complete.

## References

State machine design principles are common in distributed systems and database engines.  The above states ensure that model loading, expansion and recovery can be reasoned about and verified.  See **References.md** for relevant sources.