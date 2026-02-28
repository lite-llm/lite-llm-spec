# SPEC‑019 — Network Transport Abstraction (RDMA / NCCL / QUIC)

Lite LLM operates on clusters ranging from a few GPUs to thousands of nodes.  Each node may have different networking capabilities, from RoCE‐enabled RDMA fabrics to standard TCP/IP.  To support heterogeneity while preserving determinism and performance, the runtime abstracts network transport behind a uniform interface.  This document specifies the responsibilities of the transport layer and describes supported back‑ends: Remote Direct Memory Access (RDMA), NVIDIA Collective Communications Library (NCCL), and QUIC.

## 1 Goals

1. **Pluggable transports:** enable the runtime to swap transport mechanisms (e.g., RDMA, NCCL, QUIC) without modifying higher‑level logic.
2. **Determinism:** enforce ordered and reproducible communication semantics (see SPEC‑018) regardless of underlying transport implementation.
3. **Performance:** exploit the best available bandwidth and latency on each cluster while gracefully degrading on slower networks.
4. **Security:** allow encrypted channels for sensitive deployments (SPEC‑053).

## 2 Transport Interface

The transport interface exposes the following primitives:

* **init(rank, world size, config):** set up connections to all peers.  The `config` includes network addresses, MTU, encryption keys and selected transport type.
* **send(to, buffer, tag):** send a typed message to a peer.  The `tag` identifies the message for ordering and determinism.
* **recv(from, buffer, tag):** block until a message with the matching `tag` arrives.
* **barrier():** synchronise all ranks at a safe point.
* **allreduce(buffer, op):** perform a collective reduction using a deterministic tree or ring algorithm (SPEC‑018).
* **alltoall(send buf, recv buf):** exchange equal‑sized segments with all ranks in a deterministic order.

Each transport must implement these primitives and satisfy the semantics described in the deterministic collective spec.

## 3 Supported Transports

### 3.1 RDMA

RDMA provides low‑latency, zero‑copy data transfers over Infiniband or RoCE networks.  RDMA supports remote memory read/write without CPU involvement.  The runtime’s RDMA backend maps each expert buffer into a registered memory region and performs one‑sided writes for dispatch.  To maintain determinism, RDMA operations are serialized according to the global tag order.  StrataServe shows that pooling GPU HBM, host DRAM and SSD into a coordinated hierarchy yields high throughput when combined with RDMA peer‑to‑peer communication【75891756086750†L80-L95】.

### 3.2 NCCL

NVIDIA’s NCCL library is widely used for collective operations on NVIDIA GPUs.  It provides optimized all‑reduce, broadcast and all‑gather routines but is not always deterministic.  Lite LLM wraps NCCL calls in a deterministic scheduler that enforces a fixed reduction order and disables algorithm selection at runtime.  NCCL is used when RDMA is unavailable or when running within a single node.

### 3.3 QUIC

For cloud deployments without RDMA or GPU direct support, Lite LLM can fall back to the QUIC protocol over UDP.  QUIC supports multiplexed streams with built‑in congestion control and encryption.  The runtime uses QUIC streams for point‑to‑point messages (e.g., expert dispatch) and emulates collectives via repeated sends/receives.  While higher latency than RDMA, QUIC offers wide compatibility and can be hardened with TLS.

## 4 Implementation Considerations

* **Tag space:** assign a monotonically increasing tag to every send/receive operation to enforce order.  Tags incorporate the step, layer and routing level.
* **Connection management:** long‑lived connections reduce overhead but must handle reconnection on failure (see SPEC‑020).
* **Flow control:** avoid overwhelming slower links by implementing credit‑based flow control.
* **Security:** for encrypted deployments, wrap transport in TLS or use QUIC’s built‑in encryption.  Key material is managed by the key management system (SPEC‑057).

## 5 Future Extensions

Future specs may add support for alternative transports (e.g., UCX or custom kernel bypass).  The pluggable design allows experimentation without affecting the rest of the system.
