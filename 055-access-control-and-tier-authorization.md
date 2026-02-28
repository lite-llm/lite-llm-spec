# SPEC‑055 — Access Control & Tier Authorization

Access control ensures that only authorised users and services can invoke model operations and access specific tiers or experts.  **Tier authorisation** further restricts which tiers each tenant may activate.  This specification defines the access control mechanisms for Lite LLM.

## 1 Authentication

1. **Identity providers:** Users and services authenticate via standard mechanisms (OAuth, API keys, mTLS).  Each request carries an identity token used to look up the tenant’s permissions.
2. **Token validation:** Tokens are verified for authenticity, expiry and scope.  Invalid or expired tokens are rejected.
3. **Session binding:** An authenticated identity is bound to a session (SPEC‑050).  Subsequent streaming or batched requests use the same identity.

## 2 Authorisation

1. **Role‑based access control (RBAC):** Permissions are defined at the granularity of operations (load model, run inference, access telemetry) and tiers (hot, warm, cold, archive).
2. **Tier allowances:** Tenants may be allowed to use only certain tiers.  For example, a free tier might restrict tenants to `hot` and `warm`, while paid tenants can access `cold` and `archive`.  Attempting to load experts from unauthorised tiers results in denial.
3. **Parameter sets:** Custom fine‑tuned models or proprietary experts may be restricted to specific tenants.  Access checks ensure that other tenants cannot load them.

## 3 Integration Points

* **Model loading:** When loading a checkpoint (SPEC‑052), the runtime verifies that the tenant has permission to load the specified model and tiers.  Unauthorized models are not visible.
* **Inference:** Before activating a TierSet (SPEC‑041) or dispatching tokens, the runtime checks that the caller is authorised to use each tier and expert.  Unauthorised requests are denied or downgraded to an allowed TierSet.
* **Prefetch:** Prefetch (SPEC‑045) respects authorisation by not fetching experts from unauthorised tiers.
* **Telemetry:** Access to telemetry (SPEC‑049) is controlled; tenants may only view their own metrics.

## 4 Audit and Logging

All access control decisions are logged with the identity, requested resource, decision (allow/deny) and reason.  Logs feed into the deterministic audit system (SPEC‑056) for compliance and troubleshooting.

## 5 Security Rationale

Combining RBAC with tier authorisation prevents misuse of resources and protects proprietary content.  Memory‑safe languages like Rust help prevent accidental data exposure【443264331305965†L32-L37】, but explicit authorization is required to control who may load or execute specific tiers and experts.

By enforcing access control at every critical operation, Lite LLM upholds data privacy, contractual obligations and multi‑tenant fairness.
