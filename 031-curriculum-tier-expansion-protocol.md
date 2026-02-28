# SPEC‑031 — Curriculum Tier Expansion Protocol

Lite LLM scales capacity by adding new tiers of experts over time.  The **curriculum tier expansion protocol** defines how to gradually introduce additional tiers without destabilising training or breaking backward compatibility.  The protocol applies to both adding new experts to existing tiers and introducing entirely new tier IDs.

## 1 Rationale

Training a quadrillion‑parameter model from scratch is impractical.  Instead, we start with a smaller TierSet and progressively add tiers as training progresses and as more compute becomes available.  This curriculum approach allows the base model to converge and then leverages additional capacity for refinement.  Similar ideas appear in hierarchical gating and domain‑specialised experts【815052634867799†L135-L146】.

## 2 Phases of Expansion

1. **Preparation:** define the new tier (TierId, capacity, initial placement).  Pretrain its experts independently or initialise them randomly.
2. **Isolation:** freeze the existing tiers’ weights and router parameters.  Train only the new experts and corresponding router heads.  During this phase, the router is biased to favour existing tiers to maintain stability.
3. **Integration:** gradually increase the probability of routing tokens to the new tier.  Use a scheduling function (e.g., linear or cosine ramp) to control the activation frequency of the new tier.
4. **Joint training:** unfreeze all tiers and fine‑tune jointly with a small learning rate.  Regularise to prevent catastrophic forgetting.  Incorporate load‑balancing losses (SPEC‑032) to avoid expert collapse.

## 3 Determinism Considerations

* **Seed management:** record the seed and schedule used during expansion so that re‑training or replay reproduces the same expansion dynamics.
* **Reproducibility:** if training is resumed mid‑expansion, ensure that the router’s scheduling function picks up at the correct point in the schedule.

## 4 Compatibility and Backward Loading

Existing checkpoints remain valid after expansion.  When loading a checkpoint that lacks the new tier, the runtime simply ignores the missing tier.  Conversely, when loading a newer checkpoint with additional tiers, older runtimes must be aware that these extra tiers may be ignored if not supported.

## 5 Monitoring and Metrics

During expansion, monitor:

* **Activation frequency:** how often the new tier is selected relative to existing tiers.
* **Loss and accuracy:** watch for regressions when the new tier is integrated.
* **Expert utilisation:** ensure that the new experts are receiving sufficient gradient updates to learn.

## 6 Rollback

If the new tier causes instability or overfitting, the protocol allows rolling back to the previous TierSet.  A checkpoint prior to integration should be retained.  Reverting simply loads the previous checkpoint and disables the new tier.

By following a structured curriculum, Lite LLM can expand its capacity over the course of training while preserving stability and reproducibility.
