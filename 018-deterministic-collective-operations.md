# SPEC‑018 — Deterministic Collective Operations

Large-scale training of Lite LLM relies on collective communication primitives such as all‑reduce, broadcast and all‑gather.  These collectives must be **deterministic** so that models trained across hundreds of nodes reproduce the same results bit‑for‑bit in every run.  This specification defines the requirements, design and recommended implementation patterns for deterministic collectives.

## 1 Motivation and Overview

Distributed training synchronises gradients and parameters using collective communication.  In dense models a slight change in the ordering of floating‑point operations can lead to divergent model behaviour.  In mixture‑of‑experts training the problem is worse because sparse dispatch sends different tokens to different ranks every step.  Deterministic collectives ensure that every participating process receives and reduces the same values in the same order, eliminating non‑reproducible variation.

## 2 Requirements

### 2.1 Stable ordering

All reduction or broadcast operations **must perform reductions in a fixed, globally agreed order**.  For example, an all‑reduce should follow a tree or ring pattern where the sequence of summations is identical on every run.  Hardware‑accelerated libraries (e.g., NCCL) often produce non‑deterministic reductions due to thread races; the runtime must wrap or replace these with deterministic kernels.

### 2.2 Seed agreement

Collectives involving random number generation (e.g., dropout masking, noise injection for routing) must agree on a shared seed before the operation begins.  Seed negotiation is part of the routing consensus protocol (SPEC‑016) and ensures that any tie‑breaking noise added to routing scores is reproducible.

### 2.3 Floating‑point determinism

Floating‑point summations are not associative.  The runtime therefore fixes a reduction ordering and uses a consistent precision (e.g., fp32 or bf16 accumulation) to guarantee that the same bits are generated on every node.  When possible, intermediate results should be cast to a higher precision accumulator before being down‑cast.

## 3 Collective Operations

### 3.1 All‑reduce

The all‑reduce combines tensors from all ranks and returns the result to every rank.  Lite LLM uses all‑reduce for synchronising dense parameter gradients.  Determinism is achieved by:

1. **Tree selection:** choose a deterministic reduction tree or ring that is fixed for the lifetime of the run.
2. **Chunk ordering:** when tensors are chunked for overlapping communication and computation, the order of chunks must be consistent across ranks.
3. **Precision control:** accumulate in a higher precision and downcast to the target dtype.

### 3.2 All‑to‑all

Expert dispatch (SPEC‑015) uses an all‑to‑all pattern to exchange token activations.  Unlike all‑reduce, the payloads differ per rank; determinism depends on the routing consensus selecting an identical ordering of tokens (see SPEC‑016).  Once tokens are ordered, the transfer uses a deterministic sequence of send/recv calls.  Research on MoE communication shows that all‑to‑all traffic is highly sparse and imbalanced【529477976415087†L50-L63】; deterministic ordering prevents additional jitter from race conditions.

### 3.3 Broadcast and gather

Broadcast is used to distribute tier metadata or seeds.  The broadcast tree must be fixed and repeatable.  Gather (inverse broadcast) also uses a fixed tree and ordering.

## 4 Implementation Notes

* **Library choice:** standard libraries such as NCCL or MPI may not guarantee determinism.  The runtime should provide wrappers that enforce ordering or implement deterministic collectives in pure Rust using blocking send/recv over RDMA or QUIC.
* **Testing:** determinism tests should run the same training step multiple times on different numbers of nodes and verify identical parameter updates.
* **Performance vs determinism:** deterministic collectives may be slower than their non‑deterministic counterparts.  However, they enable reproducibility and auditing which are essential in regulated environments.

## 5 Future Work

Future specifications (e.g., SPEC‑019 and SPEC‑020) will discuss how these collectives interact with specific transport layers and fault‑tolerance mechanisms.
