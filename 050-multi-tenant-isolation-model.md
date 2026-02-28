# SPEC‑050 — Multi‑Tenant Isolation Model

Lite LLM targets shared infrastructure where multiple applications or users run models concurrently.  The **multi‑tenant isolation model** defines how requests, weights, caches and telemetry are isolated to prevent interference, data leakage and resource starvation.

## 1 Isolation Domains

1. **Session isolation:** Each request or chat session is assigned a unique **Tenant ID**.  All data structures—KV‑cache entries (SPEC‑046), routing seeds, TierSets, telemetry buffers—are keyed by this ID.
2. **Weight isolation:** Experts may be shared across tenants, but each tenant’s activation decisions remain independent.  Hot caches record which tenants loaded which experts; eviction policies ensure that one tenant cannot flush another’s prefetched experts unfairly.
3. **Compute isolation:** Scheduling of tokens to experts uses fair queueing; tokens from different tenants are interleaved at inference time but quotas limit the maximum outstanding tokens per tenant.

## 2 Resource Quotas and Limits

* **Memory quotas:** Each tenant has a configured quota on hot and warm memory usage.  When usage approaches the limit, eviction is triggered for that tenant’s least recently used KV‑cache segments or prefetched experts.
* **Bandwidth quotas:** The system may limit the rate of all‑to‑all dispatch and prefetch per tenant to prevent network congestion.
* **Compute quotas:** The number of concurrent tokens or expert calls per tenant is capped to ensure fairness.

## 3 Scheduling and Fairness

1. **Weighted fair queueing:** The dispatch engine maintains per‑tenant queues.  Round‑robin or weighted fair queueing ensures that tenants with higher priority or paid plans receive more slots, while preventing starvation of lower‑priority tenants.
2. **Back‑pressure:** If a tenant exceeds its quota, new token requests are throttled.  The runtime may respond with a `429 Too Many Requests` error or degrade to a smaller TierSet.
3. **Isolation leaks:** Care is taken to avoid cross‑tenant timing channels or side channels.  For example, prefetch performance for one tenant should not reveal which experts are hot for another tenant.

## 4 Security Controls

* **Memory safety:** By implementing the runtime in Rust with strict ownership rules, the system prevents memory corruption that could leak one tenant’s data to another【443264331305965†L32-L37】.
* **Access control:** Tenants are authenticated and authorised to use specific tiers or models (SPEC‑055).  Parameters for restricted tiers are not loaded for unauthorised tenants.
* **Encryption:** When weights are stored on disk, they are encrypted per tenant if they contain private custom fine‑tuned parameters (SPEC‑053).

## 5 Observability and Billing

* **Tenant metrics:** Telemetry (SPEC‑049) is tagged by Tenant ID.  Operators can monitor usage, latency and cost per tenant.
* **Billing:** Resource usage can be translated into cost for chargeback.  For example, prefetch bytes, expert calls and TierSet activations contribute to billing.

By providing strong isolation across sessions and tenants, Lite LLM enables secure, fair multi‑tenant deployment of massive models in shared environments, reducing risk of data leakage and resource exhaustion while maintaining performance.
