# SPEC‑057 — Secure Distributed Key Management

Lite LLM uses cryptographic keys for encrypting model tiers, signing manifests and establishing secure communications.  This specification describes the **key management service (KMS)** and protocols for secure distributed key handling.

## 1 Key Types

1. **Encryption keys:** Symmetric keys for encrypting parameter shards and optimizer states (SPEC‑053).
2. **Signature keys:** Public/private key pairs used to sign checkpoints, manifests and audit logs (SPEC‑052, SPEC‑056).
3. **TLS certificates:** Certificates and private keys for securing network connections (e.g., gRPC, HTTPS).

## 2 Key Management Service (KMS)

1. **Centralised service:** The KMS runs as a secure enclave or hardware security module (HSM).  It stores keys encrypted at rest and exposes APIs for key retrieval, generation, rotation and revocation.
2. **Authentication:** Clients authenticate to the KMS via mTLS or token‑based authentication.  Only authorised processes may request keys.
3. **Audit:** Key retrieval and usage are logged, with timestamps, requesting identity and purpose.  Logs are part of the audit trail (SPEC‑056).

## 3 Key Retrieval Protocol

1. **Request:** The runtime requests a key by providing its identity, the purpose (encryption, signing, TLS) and the specific model or tier context.
2. **Authorisation:** The KMS checks whether the requesting identity is allowed to access the requested key.
3. **Response:** The KMS returns the key (or a handle) over a secure channel.  Keys may be wrapped (encrypted under a KMS master key) and unwrapped in the secure memory of the requester.

## 4 Key Rotation

1. **Scheduled rotation:** Keys are rotated regularly.  New keys are generated, and new shards or manifests are encrypted/signed with the new keys.
2. **Phased rollout:** During rotation, old keys remain valid for decryption or verification of existing artifacts until re‑encrypted or re‑signed.
3. **Revocation:** Upon compromise, keys may be revoked immediately.  All shards encrypted with revoked keys must be re‑encrypted.

## 5 Distributed Considerations

* **Caching:** To reduce KMS calls, runtime nodes may cache decrypted keys in memory.  Cached keys are protected by in‑memory zeroization (SPEC‑054) and expiry timers.
* **Replicated KMS:** In large deployments, the KMS may run on multiple nodes with consensus (e.g., Raft) to ensure availability and consistency.
* **Network isolation:** KMS endpoints are isolated from public networks and accessible only to trusted services.

## 6 Security Assurance

Implementing a secure distributed key management system aligns with best practices for protecting sensitive cryptographic material.  Combined with memory safety in the runtime【443264331305965†L32-L37】, KMS reduces the attack surface and ensures that encryption keys cannot be leaked or misused.

By centralising key handling, Lite LLM ensures consistent application of encryption policies, enables rapid key rotation and revocation, and provides a strong foundation for secure parameter storage and communications.
