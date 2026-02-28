# SPEC‑033 — Expert Starvation Guarantees

Large mixture‑of‑experts models risk **expert starvation** when some experts seldom receive training examples.  This section establishes guarantees and mechanisms to prevent such starvation and ensure that all active experts get sufficient updates.

## 1 Minimum Update Density

Define `A_t` as the total number of expert assignments to tier `t` per training step and `M_t` as the number of experts in that tier.  To avoid starvation, the average assignments per expert (`A_t / M_t`) must exceed a minimum threshold.  If assignments per expert fall below this threshold, the runtime should either reduce the number of active experts (by lowering the top‑k values), increase the batch size or temporarily freeze some experts.

## 2 Conditional Probability Floor

The router assigns probabilities to experts.  To prevent extremely low probabilities, the system imposes a floor `delta` on the probability of selecting any expert in the active TierSet.  Practically, this is implemented by adding a small constant to the routing logits before normalising.  This ensures that even rarely chosen experts have a chance of being selected.

## 3 No‑Starvation Guarantee

If each expert has a minimum probability `delta` of being selected and the batch size is sufficiently large, the likelihood that an expert receives no assignments over many steps decays exponentially with the number of steps.  Consequently, the risk of starvation becomes vanishingly small as training proceeds, provided the minimum update density is maintained.

## 4 Interaction with Load Balancing Loss

The auxiliary load‑balancing losses (SPEC‑032) encourage the router to distribute tokens evenly across experts.  By penalising uneven assignments, these losses effectively raise the lower bound on expert usage and further reduce starvation risk.

## 5 Monitoring and Intervention

The runtime continuously tracks per‑expert assignment counts.  For experts that remain underused, it applies interventions such as:

* **Forced exploration:** temporarily boosting the router’s probability of selecting under‑used experts.
* **Expert merging:** combining multiple under‑utilised experts into one to increase effective batch size per expert.
* **Expert pruning:** removing experts that remain inactive despite interventions and reallocating their capacity to new or more useful experts.

## 6 Curriculum Considerations

During tier expansion (SPEC‑031), newly added experts may initially receive few assignments.  The curriculum protocol schedules a warm‑up phase where the new tier’s experts are sampled more frequently.  Once these experts have learned useful patterns, the router gradually reduces their sampling bias to normal levels.

By implementing minimum update density, probability floors and monitoring, Lite LLM ensures that all experts remain engaged during training and that the model fully exploits its capacity.
