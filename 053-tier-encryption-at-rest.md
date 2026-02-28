# SPEC‑053 — Tier Encryption at Rest

Model parameters and optimizer states stored in persistent tiers (warm, cold, archive) may contain proprietary intellectual property or sensitive customer‑fine‑tuned weights.  **Encryption at rest** protects these assets from unauthorised access even if the storage medium is compromised.

## 1 Scope

* **Parameters:** Expert weights, router weights and dense backbone weights stored in NVMe or object storage.
* **Optimizer states:** Sharded optimiser states (SPEC‑037) stored off device.
* **Manifests:** Checkpoint manifests may also be encrypted if they reveal model structure or tier composition.

## 2 Encryption Scheme

1. **Algorithm:** Use an industry‑standard encryption algorithm (e.g., AES‑256‑GCM) providing confidentiality and integrity.  Each shard is encrypted independently.
2. **Key management:** Keys are generated and stored in a secure key management service (KMS) (SPEC‑057).  Each tier or model may use a separate key, supporting key rotation and per‑customer isolation.
3. **Initialization vectors:** IVs are unique per shard and stored alongside the ciphertext.  IV reuse is prohibited.
4. **Authentication tag:** Each encrypted shard includes an authentication tag.  Verification of the tag precedes decryption.

## 3 Operational Workflow

1. **Encryption on write:** When creating a checkpoint or storing a shard, the runtime encrypts the data using the appropriate key and writes the ciphertext to persistent storage.  Plaintext is zeroised after encryption (SPEC‑054).
2. **Decryption on read:** Upon loading a shard, the runtime retrieves the key from the KMS, verifies the authentication tag and decrypts the data into memory.  The key is held only temporarily and cleared once decryption completes.
3. **Cache layering:** Hot caches (HBM) typically store plaintext weights; encryption/decryption only occurs when moving to/from persistent tiers.  Warm tiers may optionally store plaintext if they are in trusted memory domains.

## 4 Key Rotation and Revocation

* **Rotation:** Keys may be rotated periodically or on compromise.  During rotation, new shards are encrypted with the new key.  Old encrypted shards remain decryptable until re‑encrypted or expired.
* **Revocation:** If a tenant is revoked from a tier, the key for that tenant’s encrypted shards can be invalidated.  Without the key, the shards are useless.

## 5 Compliance

Encryption at rest helps satisfy regulatory requirements and security best practices.  The U.S. government’s report on memory‑safe languages emphasises the importance of languages and systems that embed safety mechanisms【443264331305965†L32-L37】.  Combining memory safety with encryption at rest enhances overall system security.

By encrypting tiers at rest, Lite LLM protects confidential model weights against data breaches and ensures that unauthorised parties cannot read stored parameters even if storage devices are compromised.
