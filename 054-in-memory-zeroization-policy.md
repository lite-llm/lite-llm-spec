# SPEC‑054 — In‑Memory Zeroization Policy

Security best practices require clearing sensitive data from memory when it is no longer needed.  The **in‑memory zeroization policy** outlines how Lite LLM zeroes out model weights, keys and caches to reduce the risk of data leakage or exploitation.

## 1 Sensitive Data

* **Decryption keys:** Keys retrieved from the KMS for decrypting shards (SPEC‑053).
* **Plaintext weights:** Decrypted expert weights, router tables and dense parameters, especially if loaded from encrypted tiers.
* **Optimizer states:** Moment estimates and other derivative data that may reveal model internals.
* **User content:** Token embeddings and hidden states in the KV‑cache (SPEC‑046) that may contain sensitive user data.

## 2 Zeroization Events

1. **After decryption:** Immediately after verifying authentication tags and decrypting a shard, the decryption key is zeroised in memory.  Keys are not cached beyond the scope of decryption.
2. **Cache eviction:** When a KV‑cache segment or expert weight is evicted from hot memory, the memory region is overwritten with zeros before being returned to the allocator.  This prevents residual data from being accessed by another tenant or process.
3. **Session termination:** When a streaming session ends (SPEC‑047), all associated buffers, hidden states and seeds are cleared.
4. **Checkpoint overwrite:** When updating a checkpoint, any replaced memory is zeroised before reallocation.

## 3 Implementation Mechanisms

* **Secure memset:** Use a memory‑secure zeroisation function that is not optimised away by the compiler.  In Rust, libraries like `zeroize` provide primitives to zero arrays securely.
* **Guarded regions:** Memory regions containing keys or plaintext are annotated to ensure that debugging tools do not inadvertently log them.
* **Hardware support:** On some platforms, hardware instructions (e.g., `clflushopt`) flush caches or zero memory securely.

## 4 Audit and Compliance

* **Logging:** Zeroisation events may be logged with timestamps and identifiers (e.g., tenant, session) in the deterministic audit log (SPEC‑056).  Logs record that zeroisation occurred, but not the data content.
* **Verification:** Memory scanning tools during audits can verify that freed memory does not contain residual sensitive data.

## 5 Security Rationale

The memory‑safe language report notes that languages like Rust embed protections against memory safety issues【443264331305965†L32-L37】; however, memory safety alone does not guarantee confidentiality of data in memory.  Zeroisation policies complement Rust’s safety features by ensuring that sensitive data cannot be recovered after use.

By rigorously zeroising sensitive data, Lite LLM mitigates the risk of data exposure through memory dumps or reuse of uninitialised memory, providing an additional layer of protection for users and model owners.
