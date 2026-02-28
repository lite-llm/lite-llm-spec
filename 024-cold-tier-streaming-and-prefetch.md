# SPEC‑024 — Cold Tier Streaming & Prefetch

The **cold tier** consists of large capacity NVMe solid state drives and, optionally, network‑attached block devices.  Experts stored in the cold tier are rarely used but must be accessible on demand.  This specification defines how cold experts are streamed into memory and how the system prefetches data to hide I/O latency.

## 1 Characteristics of the Cold Tier

* **High capacity:** NVMe provides terabytes of storage per node, sufficient to hold trillions of parameters.
* **High latency:** random access incurs microsecond‑scale latency; sequential reads are faster.
* **Asynchronous I/O:** streaming must overlap with computation to avoid stalling kernels.  StrataServe demonstrates a four‑stage pipeline that overlaps SSD I/O with compute and network transfers【75891756086750†L80-L95】.

## 2 Streaming Protocol

1. **Chunking:** large expert tensors are stored as contiguous chunks on NVMe.  Each chunk includes a header with version and checksum.
2. **Asynchronous read:** the runtime submits read requests via an asynchronous I/O interface (e.g., io_uring) and returns control to the scheduler.
3. **DMA staging:** upon completion, the data is moved to pinned DRAM (warm tier) for potential promotion.
4. **Verification:** the checksum is verified; if mismatched, the runtime retries the read or fetches from a replica.

## 3 Prefetching

Given the latency of cold tier reads, the runtime must prefetch experts ahead of their activation:

* **Prediction models:** the predictive prefetch engine monitors routing statistics and uses heuristics or machine‑learning models to predict which experts will be activated in the near future.  For example, if a token’s routing history shows a pattern of adjacent experts, the engine prefetches them.
* **Prewarm window:** define a sliding time window (e.g., 2–5 future inference steps) during which predictions are made.  Prefetch requests are issued for experts likely to be used within that window.
* **I/O scheduling:** prefetch requests have lower priority than demand loads but may be cancelled if capacity is needed for demand.

## 4 Backpressure and Throttling

Reading from NVMe too aggressively can saturate the I/O subsystem and starve demand loads.  The runtime employs backpressure: if the queue length exceeds a threshold or the warm tier is near capacity, prefetching is slowed or paused.

## 5 Integration with Archive Tier

Cold tier streaming is the last stage before data is fetched from remote object stores (SPEC‑025).  If a cold tier read fails repeatedly or the expert is absent, the runtime escalates the fetch to the archive tier.

Through asynchronous streaming and intelligent prefetching, the cold tier provides a scalable reservoir for seldom‑used parameters while keeping latency within tolerable bounds.
