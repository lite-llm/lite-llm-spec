# SPEC‑047 — Streaming Inference Protocol

For interactive applications, Lite LLM must deliver tokens as soon as they are ready rather than waiting for the full sequence.  The **streaming inference protocol** defines how to process incremental inputs, emit outputs token by token and manage resources across long streams.

## 1 Streaming Workflow

1. **Session initiation:** A client starts an inference session by sending an initial prompt.  The runtime allocates a KV‑cache (SPEC‑046) and selects an initial TierSet (SPEC‑041).
2. **Incremental token generation:** For each step:
   1. Compute attention using existing KV‑cache entries.
   2. Run the router and expert dispatch (SPEC‑044) with dynamic prefetching (SPEC‑045).
   3. Sample or choose the next token.
   4. Append the new token to the KV‑cache and emit it to the client.
3. **Token streaming:** The runtime flushes tokens to the client as soon as they are available.  Back‑pressure from the client throttles the generator.
4. **Continued input:** The client may send new user input mid‑stream (e.g., chat context).  The runtime concatenates this input, updates the KV‑cache and resumes generation.

## 2 Resource Management

* **KV‑cache lifetime:** The KV‑cache persists for the duration of the session.  When a stream ends, the cache is cleared.
* **Tier prefetch:** Prefetching continues to anticipate upcoming expert usage; unused prefetched weights may be retained if subsequent tokens arrive quickly.
* **Budget adjustments:** The TierSet may change mid‑stream based on latency or cost feedback (SPEC‑041).  For example, if the user requests more detail, the runtime can enable additional tiers.
* **Heartbeat:** The client may send heartbeats to indicate that it is still connected.  If heartbeats stop, the session may time out and free resources.

## 3 Determinism & Consistency

1. **Ordering:** The stream preserves causal order: each output token depends only on prior inputs and internal state.  Concurrency within the runtime must not reorder tokens.
2. **Seed management:** The random seed for sampling and routing is fixed per session.  Changing tiers or budgets does not re‑seed the generator, so repeated runs with the same settings are reproducible.
3. **Idempotency:** If a token is re‑requested (e.g., due to network retry), the runtime must return the same token sequence up to that point.

## 4 Error Handling

* **Transient errors:** Network interruptions or intermediate timeouts are retried.  The runtime buffers a small window of recent tokens to resend if necessary.
* **Fatal errors:** On unrecoverable failures (hardware fault, memory corruption), the stream is closed with an error code.  The client may restart a new session.
* **Graceful cancellation:** The client can cancel the stream; the runtime stops generation, clears caches and releases prefetch in progress.

## 5 Integration with Telemetry and Policies

1. **Metrics:** The runtime records per‑token latency, bytes read/written, tier hits, expert hits and prefetch efficiency, and feeds them into the inference telemetry model (SPEC‑049).
2. **Adaptive control:** The latency budget solver (SPEC‑042) can adjust sampling temperature or TierSet mid‑stream if the runtime is exceeding latency or cost budgets.
3. **Compliance:** The streaming protocol must support logging and compliance requirements (SPEC‑059), including per‑session audit logs (SPEC‑056).

By delivering tokens incrementally and adapting TierSets and prefetching on the fly, the streaming inference protocol enables interactive chat and real‑time applications while respecting deterministic guarantees and resource constraints.
