# Lite LLM – References

This document contains all academic and technical references cited in the Lite LLM white paper.

---

## Transformer Foundations

[1] Vaswani, A., et al. (2017).  
"Attention Is All You Need."  
Advances in Neural Information Processing Systems (NeurIPS).

[2] Dai, Z., et al. (2019).  
"Transformer-XL: Attentive Language Models Beyond a Fixed-Length Context."  
ACL.

[3] Su, J., et al. (2021).  
"RoFormer: Enhanced Transformer with Rotary Position Embedding."  
arXiv:2104.09864.

---

## Scaling Laws

[4] Kaplan, J., et al. (2020).  
"Scaling Laws for Neural Language Models."  
arXiv:2001.08361.

[5] Hoffmann, J., et al. (2022).  
"Training Compute-Optimal Large Language Models."  
arXiv:2203.15556.

[6] Henighan, T., et al. (2020).  
"Scaling Laws for Autoregressive Generative Modeling."  
arXiv:2010.14701.

---

## Mixture-of-Experts (MoE)

[7] Shazeer, N., et al. (2017).  
"Outrageously Large Neural Networks: The Sparsely-Gated Mixture-of-Experts Layer."  
ICLR.

[8] Lepikhin, D., et al. (2020).  
"GShard: Scaling Giant Models with Conditional Computation and Automatic Sharding."  
arXiv:2006.16668.

[9] Fedus, W., Zoph, B., & Shazeer, N. (2021).  
"Switch Transformers: Scaling to Trillion Parameter Models with Simple and Efficient Sparsity."  
JMLR.

[10] Du, N., et al. (2022).  
"GLaM: Efficient Scaling of Language Models with Mixture-of-Experts."  
ICML.

[11] Zoph, B., & Le, Q. (2017).  
"Neural Architecture Search with Reinforcement Learning."  
ICLR.

---

## Hierarchical Routing & Conditional Computation

[12] Eigen, D., et al. (2013).  
"Learning Factored Representations in a Deep Mixture of Experts."  
ICLR Workshop.

[13] Jordan, M., & Jacobs, R. (1994).  
"Hierarchical Mixtures of Experts and the EM Algorithm."  
Neural Computation.

[14] Roller, S., et al. (2021).  
"Hash Layers for Large Sparse Models."  
NeurIPS.

[15] Lewis, P., et al. (2020).  
"Retrieval-Augmented Generation for Knowledge-Intensive NLP Tasks."  
NeurIPS.

---

## Load Balancing and Routing Stability

[16] Shazeer, N. (2018).  
"Adafactor: Adaptive Learning Rates with Sublinear Memory Cost."  
ICML.

[17] Roller, S., et al. (2021).  
"Sparse Mixture of Experts are Robust Multi-Task Learners."  
arXiv.

[18] Hazimeh, H., et al. (2021).  
"FastMoE: A Fast Mixture-of-Expert Training System."  
MLSys.

[19] Zhou, Y., et al. (2022).  
"Mixture-of-Experts with Expert Choice Routing."  
NeurIPS.

---

## Distributed Systems & Communication Complexity

[20] Sergeev, A., & Del Balso, M. (2018).  
"Horovod: Fast and Easy Distributed Deep Learning in TensorFlow."  
arXiv:1802.05799.

[21] Patarasuk, P., & Yuan, X. (2009).  
"Bandwidth Optimal All-reduce Algorithms for Clusters of Workstations."  
Journal of Parallel and Distributed Computing.

[22] Thakur, R., Rabenseifner, R., & Gropp, W. (2005).  
"Optimization of Collective Communication Operations in MPICH."  
International Journal of High Performance Computing Applications.

---

## Determinism & Numerical Stability

[23] Higham, N. (2002).  
"Accuracy and Stability of Numerical Algorithms."  
SIAM.

[24] Demmel, J., et al. (2013).  
"Reproducible Floating Point Computations."  
SC Conference.

[25] IEEE (2019).  
"IEEE Standard for Floating-Point Arithmetic (IEEE 754-2019)."

---

## Optimization & Convergence Theory

[26] Bottou, L., Curtis, F., & Nocedal, J. (2018).  
"Optimization Methods for Large-Scale Machine Learning."  
SIAM Review.

[27] Robbins, H., & Monro, S. (1951).  
"A Stochastic Approximation Method."  
Annals of Mathematical Statistics.

[28] Boucheron, S., Lugosi, G., & Massart, P. (2013).  
"Concentration Inequalities."  
Oxford University Press.

---

## Storage Hierarchy & Systems Architecture

[29] Hennessy, J., & Patterson, D. (2017).  
"Computer Architecture: A Quantitative Approach."  
Morgan Kaufmann.

[30] Dean, J., & Barroso, L. (2013).  
"The Tail at Scale."  
Communications of the ACM.

[31] Facebook AI Research (2021).  
"ZeRO: Memory Optimization Towards Training Trillion Parameter Models."  
arXiv:1910.02054.

---

## Scaling Beyond Trillion Parameters

[32] Narayanan, D., et al. (2021).  
"Efficient Large-Scale Language Model Training on GPU Clusters Using Megatron-LM."  
SC.

[33] Rajbhandari, S., et al. (2020).  
"ZeRO-Offload: Democratizing Billion-Scale Model Training."  
USENIX ATC.

[34] Li, S., et al. (2023).  
"DeepSpeed-MoE: Advancing Mixture-of-Experts Inference and Training to Power Next-Generation AI Scale."  
ICML.

---

End of References.