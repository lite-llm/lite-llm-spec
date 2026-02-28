# SPEC‑032 — Load Balancing Loss Formalisation

Sparse mixture‑of‑experts models can suffer from **expert collapse**, where a few experts handle most tokens while others remain idle.  To encourage balanced utilisation and improve generalisation, Lite LLM employs auxiliary load‑balancing losses at each routing level.  This specification formalises these losses without relying on specialised mathematical notation.

## 1 Problem Statement

For a given batch of tokens, the router computes a probability distribution over tiers, groups and experts for each token (see SPEC‑005).  Unbalanced routing leads to poor expert specialisation and harms downstream tasks.  The goal of load‑balancing loss is to penalise deviations from uniform usage while respecting capacity constraints.

## 2 Loss Definitions

At each routing level, we compute the empirical usage distribution and compare it to the uniform distribution over eligible options.  For example:

* **Tier level:** compute the average probability that a token selects each tier.  Penalise tiers whose usage is far from equal by computing a Kullback–Leibler (KL) divergence between the empirical distribution and a uniform distribution over the active tiers.
* **Group level:** for each tier, compute the average probability that a token selects each group.  Penalise deviations from equal usage across groups in that tier.
* **Expert level:** within each selected group, compute the average probability assigned to each expert.  Penalise deviations from equal usage across experts.

We then combine these penalties with tunable weights for each level (e.g., α_t for tiers, α_g for groups and α_e for experts) to form a total auxiliary loss.  This encourages the router to distribute tokens more evenly across tiers, groups and experts while still learning task‑relevant specialisations.  The approach generalises regularisation methods such as the Z‑loss used in Switch Transformers【815052634867799†L135-L146】.

## 3 Implementation Considerations

* **Computing marginals:** computing the empirical distributions requires summing routing probabilities across the batch.  In data‑parallel training, these sums must be aggregated across workers via an all‑reduce.
* **Scaling weights:** the weights α can be tuned based on the number of options at each level.  Higher tiers may tolerate more imbalance due to capacity constraints.
* **Annealing:** start with larger α values to encourage exploration and reduce them as training stabilises.

## 4 Relation to Capacity and Activation

The auxiliary loss interacts with the active compute bound defined in SPEC‑006.  Balanced routing helps ensure that enough experts receive updates to remain useful.  Without balancing, some experts may starve and underfit, while others become overloaded【529477976415087†L50-L63】.  Balanced utilisation also improves training stability and reduces overfitting to a few experts.

## 5 Monitoring and Adaptive Tuning

The runtime monitors per‑expert activation counts and the value of the auxiliary loss during training.  If certain experts remain underutilised, the corresponding weights can be increased.  Conversely, if balancing harms performance by sending too many tokens to suboptimal experts, weights can be reduced.  Automated tuning of α values may be incorporated into the optimizer in future revisions.

By formalising load balancing losses, Lite LLM prevents expert collapse and promotes fair utilisation across a vast parameter space, improving generalisation and training stability.
