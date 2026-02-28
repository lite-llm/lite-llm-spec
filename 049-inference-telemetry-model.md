# SPEC‑049 — Inference Telemetry Model

Operational excellence requires visibility into the behaviour of the inference runtime.  Lite LLM includes a comprehensive **telemetry model** for capturing metrics, events and traces during inference.  These data power adaptive control (SPEC‑042), cost‑adaptive routing (SPEC‑048), dynamic prefetch (SPEC‑045) and load balancing.

## 1 Metric Categories

### 1.1 Latency

* **Per‑token latency:** Time from routing invocation to token emission, broken down into compute (attention + expert) and I/O (prefetch/dispatch) components.
* **End‑to‑end latency:** Time from request receipt to final token emission.
* **Tier latency:** Average time to fetch an expert from hot/warm/cold tiers, capturing the benefit of hierarchical storage【75891756086750†L80-L95】.

### 1.2 Resource Utilisation

* **Memory usage:** Hot, warm and cold usage; KV‑cache size (SPEC‑046); occupancy per tier.
* **Bandwidth:** Bytes sent/received during expert dispatch (SPEC‑044); prefetch bytes (SPEC‑045).
* **Compute utilisation:** GPU/CPU utilisation per layer and per expert.

### 1.3 Routing and Usage

* **Tier hit rates:** Frequency of expert activations per tier; cold‑miss rates.
* **Expert hit distribution:** Histogram of expert usage; measures load balance and identifies under/over‑used experts.
* **Prefetch effectiveness:** Ratio of prefetched data actually used; wasted prefetch bytes.

### 1.4 Errors and Warnings

* **Routing mismatches:** Instances where ranks disagree on the active experts (SPEC‑008).
* **Prefetch failures:** Failed or delayed I/O operations.
* **Cache evictions:** Forced evictions due to memory pressure.
* **Security violations:** Access control denials (SPEC‑055), integrity check failures (SPEC‑052).

## 2 Data Collection and Aggregation

1. **Sampling:** To minimise overhead, the runtime samples telemetry at configurable intervals (e.g., every N tokens) and aggregates metrics over mini‑batches.
2. **Local aggregation:** Each rank aggregates its metrics locally and emits periodic summaries.  For multi‑node deployments, a lightweight aggregator collects and merges summaries.
3. **Trace IDs:** Requests carry unique trace IDs linking metrics across layers and services.  This supports latency root cause analysis.
4. **Persistent storage:** Telemetry is written to a metrics backend (e.g., Prometheus, OpenTelemetry).  Sensitive data is anonymised or redacted according to compliance policies (SPEC‑059).

## 3 Integration with Control Loops

1. **Latency solver:** The solver uses telemetry to estimate current latency and adjust TierSets or sampling parameters (SPEC‑042).
2. **Cost‑adaptive routing:** Telemetry supplies cost estimates for experts and tiers (SPEC‑048).  Over time the system refines cost models using observed latency, memory and energy.
3. **Dynamic prefetching:** Prefetch heuristics (SPEC‑045) rely on hit/miss rates and wasted bytes to tune lookahead windows and priorities.
4. **Load balancing:** Routing algorithms (SPEC‑032) use expert hit distributions to update load balancing losses and avoid expert collapse.

## 4 Security and Privacy

* **Data minimisation:** Only metrics necessary for optimisation are recorded.  User content and token values are not stored.
* **Access control:** Telemetry endpoints require authentication.  Sensitive metrics (e.g., memory addresses) are masked.
* **Regulatory compliance:** The telemetry model supports data retention policies and deletion requests (SPEC‑059).

By collecting and acting on rich telemetry, Lite LLM adapts its routing, TierSet and prefetch strategies to meet latency and cost requirements and ensures fair use of expert resources while maintaining user privacy and security.
