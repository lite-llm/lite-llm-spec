# Lite LLM Enterprise Runtime Specification 009: Concurrency & Threading Model

## Purpose

This specification describes how the runtime schedules tasks and utilizes threads.  It covers asynchronous execution, task scheduling, expert execution queues and lock‑free data structures to maximize throughput on modern multi‑core and multi‑GPU systems.

## Asynchronous Runtime

The runtime is built on an asynchronous task executor.  Each stage of the computation pipeline (tokenization, attention, routing, expert execution, communication, etc.) is represented as an asynchronous task.  Tasks are scheduled onto worker threads according to priority and dependencies.  The executor must support:

* **Fair scheduling:** Prevent starvation of low‑priority tasks.
* **Work stealing:** Idle threads may steal tasks from others, balancing load.
* **Back‑pressure:** When downstream queues fill up (e.g., due to slow I/O), upstream tasks slow down to avoid memory blowups.

## Task Scheduling

Tasks are grouped into pipelines corresponding to the layers of the transformer.  For each batch, tasks flow through these pipelines:

1. **Token Preprocessing:** Embed tokens and apply positional encoding.
2. **Attention Kernel Tasks:** Run matrix multiplications and softmax for each attention head.  These tasks can be parallelized across tensor parallel ranks.
3. **Routing Tasks:** Compute routing scores and perform top‑$k$ selection.  These tasks are CPU‑bound and may run on separate worker threads.
4. **Expert Execution Tasks:** For each selected expert, schedule an MLP forward pass on the device where the expert resides.  These tasks may wait for all‑to‑all communication to complete.
5. **Aggregation Tasks:** Collect expert outputs and combine them according to routing weights.

The scheduler ensures that dependent tasks do not execute until their prerequisites complete.  Futures/promises can be used to manage dependencies.

## Expert Execution Queues

Each expert (or group of experts on a device) has an associated execution queue.  When tokens are assigned to the expert, a work item describing the batch of token slices is enqueued.  A dedicated thread or thread pool drains this queue and runs the expert MLP.  Key properties:

* **Batching:** To improve efficiency, tokens destined for the same expert are batched together.  Batching reduces kernel launch overhead and increases arithmetic intensity.
* **Concurrency:** Multiple experts on the same device can be executed concurrently using streams (CUDA) or command queues (ROCm) if resources allow.
* **Prefetching:** The scheduler may issue prefetch commands for expert weights from DRAM/NVMe into HBM ahead of execution.

## Lock‑Free Data Structures

To minimize contention, the runtime uses lock‑free queues and atomic counters:

* **MPSC Queues:** Multi‑producer, single‑consumer queues for expert execution tasks allow multiple routing threads to enqueue work without blocking.
* **Atomic Reference Counts:** `Arc` (Atomic Reference Count) is used to manage tensor lifetimes safely across threads.
* **Ring Buffers:** For repeated communication patterns (e.g., KV‑cache updates), ring buffers avoid expensive allocations and deallocations.

## References

The design borrows from asynchronous runtimes like Tokio and async‑std in Rust, as well as kernel scheduling strategies in deep learning frameworks.  See **References.md** for further reading.