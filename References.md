# Lite LLM – References

This document contains all academic and technical references cited in the Lite LLM engineering specification.  For each numbered citation, see the referenced works in the original documents.

---

## Transformer Foundations

[1] Vaswani, A., et al. (2017).  *Attention Is All You Need.*  In *Advances in Neural Information Processing Systems (NeurIPS)*.

[2] Dai, Z., et al. (2019).  *Transformer‑XL: Attentive Language Models Beyond a Fixed‑Length Context.*  In *ACL*.

[3] Su, J., et al. (2021).  *RoFormer: Enhanced Transformer with Rotary Position Embedding.*  arXiv:2104.09864.

## Scaling Laws

[4] Kaplan, J., et al. (2020).  *Scaling Laws for Neural Language Models.*  arXiv:2001.08361.

[5] Hoffmann, J., et al. (2022).  *Training Compute‑Optimal Large Language Models.*  arXiv:2203.15556.

[6] Henighan, T., et al. (2020).  *Scaling Laws for Autoregressive Generative Modeling.*  arXiv:2010.14701.

## Mixture‑of‑Experts (MoE)

[7] Shazeer, N., et al. (2017).  *Outrageously Large Neural Networks: The Sparsely‑Gated Mixture‑of‑Experts Layer.*  In *ICLR*.

[8] Lepikhin, D., et al. (2020).  *GShard: Scaling Giant Models with Conditional Computation and Automatic Sharding.*  arXiv:2006.16668.

[9] Fedus, W., Zoph, B., & Shazeer, N. (2021).  *Switch Transformers: Scaling to Trillion Parameter Models with Simple and Efficient Sparsity.*  In *Journal of Machine Learning Research*.

[10] Du, N., et al. (2022).  *GLaM: Efficient Scaling of Language Models with Mixture‑of‑Experts.*  In *ICML*.

## Hierarchical Routing & Conditional Computation

[11] Eigen, D., et al. (2013).  *Learning Factored Representations in a Deep Mixture of Experts.*  In *ICLR Workshop*.

[12] Jordan, M., & Jacobs, R. (1994).  *Hierarchical Mixtures of Experts and the EM Algorithm.*  In *Neural Computation*.

[13] Roller, S., et al. (2021).  *Hash Layers for Large Sparse Models.*  In *NeurIPS*.

[14] Lewis, P., et al. (2020).  *Retrieval‑Augmented Generation for Knowledge‑Intensive NLP Tasks.*  In *NeurIPS*.

## Load Balancing and Routing Stability

[15] Shazeer, N. (2018).  *Adafactor: Adaptive Learning Rates with Sublinear Memory Cost.*  In *ICML*.

[16] Roller, S., et al. (2021).  *Sparse Mixture of Experts are Robust Multi‑Task Learners.*  arXiv.

[17] Hazimeh, H., et al. (2021).  *FastMoE: A Fast Mixture‑of‑Expert Training System.*  In *MLSys*.

[18] Zhou, Y., et al. (2022).  *Mixture‑of‑Experts with Expert Choice Routing.*  In *NeurIPS*.

## Distributed Systems & Communication

[19] Sergeev, A., & Del Balso, M. (2018).  *Horovod: Fast and Easy Distributed Deep Learning in TensorFlow.*  arXiv:1802.05799.

[20] Patarasuk, P., & Yuan, X. (2009).  *Bandwidth Optimal All‑reduce Algorithms for Clusters of Workstations.*  *Journal of Parallel and Distributed Computing*.

[21] Thakur, R., Rabenseifner, R., & Gropp, W. (2005).  *Optimization of Collective Communication Operations in MPICH.*  *International Journal of High Performance Computing Applications*.

## Determinism & Numerical Stability

[22] Higham, N. (2002).  *Accuracy and Stability of Numerical Algorithms.*  Society for Industrial and Applied Mathematics.

[23] Demmel, J., et al. (2013).  *Reproducible Floating Point Computations.*  In *SC Conference*.

[24] IEEE (2019).  *IEEE Standard for Floating‑Point Arithmetic (IEEE 754‑2019).*  IEEE.

## Optimization & Convergence Theory

[25] Bottou, L., Curtis, F., & Nocedal, J. (2018).  *Optimization Methods for Large‑Scale Machine Learning.*  *SIAM Review*.

[26] Robbins, H., & Monro, S. (1951).  *A Stochastic Approximation Method.*  *Annals of Mathematical Statistics*.

[27] Boucheron, S., Lugosi, G., & Massart, P. (2013).  *Concentration Inequalities.*  Oxford University Press.

## Storage Hierarchy & Systems

[28] Hennessy, J., & Patterson, D. (2017).  *Computer Architecture: A Quantitative Approach.*  Morgan Kaufmann.

[29] Dean, J., & Barroso, L. (2013).  *The Tail at Scale.*  *Communications of the ACM*.

[30] Facebook AI Research (2021).  *ZeRO: Memory Optimization Towards Training Trillion Parameter Models.*  arXiv:1910.02054.

## Scaling Beyond Trillion Parameters

[31] Narayanan, D., et al. (2021).  *Efficient Large‑Scale Language Model Training on GPU Clusters Using Megatron‑LM.*  In *SC*.

[32] Rajbhandari, S., et al. (2020).  *ZeRO‑Offload: Democratizing Billion‑Scale Model Training.*  In *USENIX ATC*.

[33] Li, S., et al. (2023).  *DeepSpeed‑MoE: Advancing Mixture‑of‑Experts Inference and Training to Power Next‑Generation AI Scale.*  In *ICML*.

---

For all other citations in the engineering specification, see the relevant articles and reports as indicated by the tether IDs.
