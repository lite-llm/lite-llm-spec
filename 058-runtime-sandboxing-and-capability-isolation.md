# SPEC‑058 — Runtime Sandboxing & Capability Isolation

Lite LLM’s runtime loads and executes user‑supplied code, such as custom routing functions or expert modules.  To prevent malicious or buggy extensions from compromising the system, the runtime uses **sandboxing** and **capability isolation**.

## 1 Threat Model

1. **Malicious plugins:** Untrusted code may attempt to access sensitive memory, read other tenants’ data or perform side‑effects.
2. **Buggy code:** Even non‑malicious code may contain bugs that lead to crashes, deadlocks or memory corruption.
3. **Third‑party libraries:** Integration with external frameworks or optimized kernels may introduce unsafe behaviour.

## 2 Sandboxing Techniques

1. **Language safety:** Rust ensures memory safety and prevents many classes of bugs【443264331305965†L32-L37】.  Plugin boundaries use Rust traits that restrict what operations are allowed.
2. **Process isolation:** Untrusted modules run in separate processes or WebAssembly sandboxes.  Communication with the main runtime occurs over restricted channels.
3. **Capability tokens:** Plugins are granted explicit capabilities (e.g., read a specific tensor, call specific API) via tokens.  The runtime enforces capability checks on each API call.
4. **System call filtering:** Using seccomp or similar mechanisms, plugin processes are restricted to a minimal set of system calls.  File system and network access are denied unless specifically authorised.
5. **Resource limits:** CPU, memory and I/O limits are enforced per plugin to prevent denial‑of‑service.

## 3 Capability Model

1. **Least privilege:** Plugins receive only the capabilities necessary to perform their function.  For example, a custom router receives the hidden state tensor and may return routing decisions, but cannot access the KV‑cache or underlying weights.
2. **Time‑bounded:** Capabilities may expire after a time interval or when the session ends.  The runtime revokes them automatically.
3. **Auditable:** Capability grants and usage are logged in the audit system (SPEC‑056).

## 4 Integration Points

* **Custom routing functions:** Users may supply a function to compute routing logits.  The function runs inside a sandbox, with access only to the input tensor and configuration parameters.  The output is validated for dimensionality and value range.
* **Expert plug‑ins:** Organisations may supply proprietary experts compiled to WebAssembly.  The runtime loads them into a sandbox, passing tensors via shared memory.  Results are validated before use.
* **External kernels:** GPU or network kernels implemented in C/C++/CUDA are invoked through FFI wrappers with safety checks.  Only whitelisted functions may be called.

## 5 Security Rationale

Sandboxing and capability isolation complement memory‑safe language guarantees by containing untrusted extensions.  They prevent a compromised plugin from reading memory belonging to other tenants or modifying global state.  Combined with encryption, access control and audit logging, they provide a defence‑in‑depth strategy.

By adopting robust sandboxing, Lite LLM enables extensibility (custom routers, experts) without sacrificing security or stability.
