# Lite LLM Enterprise Runtime Specification 015: All‑to‑All Communication Protocol

## Purpose

This specification defines the all‑to‑all communication protocol used in expert parallelism.  The protocol supports dispatching token data to remote experts and returning results.  It must be efficient, deterministic and fault tolerant.

## Definitions

* **Ranks:** There are $R$ expert parallel ranks.  Each rank owns a subset of experts.
* **Messages:** Each rank sends and receives a message to and from every other rank.  The message contains token embeddings destined for experts on that rank.
* **Payload Size:** If rank $i$ sends $m_{i,j}$ token embeddings to rank $j$, the total payload is $m_{i,j} \times d$ elements, where $d$ is the hidden dimension.

## Protocol Steps

1. **Packing Phase:** Each rank identifies tokens assigned to experts on each destination rank.  It packs these tokens into contiguous buffers.  Each buffer begins with a header indicating the number of tokens and their indices for ordering.
2. **Transfer Phase:** Use an all‑to‑all collective to exchange buffers.  Each rank sends its packed buffer to every other rank.  Communication libraries such as NCCL, MPI or custom RDMA transports can be used.
3. **Unpacking & Execution Phase:** Each rank receives buffers from all peers.  It unpacks tokens, groups them by expert and executes the experts.  Execution may happen concurrently.
4. **Return Phase:** After computing outputs, each rank packs results into return buffers.  Another all‑to‑all exchanges these outputs back to the originating ranks.
5. **Aggregation Phase:** Each rank receives returned outputs and combines them according to the routing weights (e.g., summation or weighted sum).

The protocol repeats for each MoE layer and for forward and backward passes during training.

## Determinism & Ordering

To ensure deterministic operation:

* The order of tokens within each buffer must be consistent across runs.  Tokens are sorted by their original index before packing.
* Headers include token indices to restore order on the receiving side.
* The choice of communication library must support deterministic collectives (SPEC 018).  Some libraries (e.g., NCCL) guarantee deterministic behaviour for certain operations.

## Fault Tolerance

* **Timeouts:** Transfers include timeouts.  If a rank fails to send data within the timeout, the protocol aborts and triggers recovery (SPEC 020).
* **Checksum:** Each buffer includes a checksum.  Mismatches indicate data corruption and cause abort.
* **Rebroadcast:** If a rank detects a failure, remaining ranks may reduce the group size and resend data.  The rerouting mechanism (SPEC 014) masks out failed ranks and redistributes experts.

## Implementation Considerations

* **Granularity:** The number of tokens per message affects overhead.  Batching many small tokens yields fewer messages but might increase per‑message latency.  Adaptive batching can optimize this trade‑off.
* **Network Topology:** On networks with hierarchical topologies, implement hierarchical all‑to‑all (e.g., intra‑node then inter‑node) to reduce congestion.
* **Pipelining:** Overlap packing, communication and execution to hide latency.  For example, while waiting for data from rank $i$, process data from rank $j$.

## References

The protocol is informed by communication patterns in GShard, DeepSpeed‑MoE and parameter servers.  See **References.md** for further reading.