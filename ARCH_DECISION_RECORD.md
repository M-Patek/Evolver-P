# Evolver Architecture Decision Record (ADR)

**Status:** v1.0 Stable

This document records the critical architectural decisions that define Evolver v1.0.

---

## 1. The Foundation

### ADR-001: The Trinity Architecture
* **Decision:** Decouple the system into **Soul** (Algebra), **Will** (Optimization), and **Body** (Projection).
* **Rationale:** Allows swapping mathematical cores without rewriting the optimizer or logic adapters.

### ADR-002: Native Logic Generation
* **Decision:** Evolver is a generator, not a corrector. It grows logic paths from an algebraic seed rather than patching LLM output.

---

## 2. The Algebraic Core (v1.0 Definition)

### ADR-003: Definite Quaternion Algebras (Pizer Graphs)
* **Context:** Early prototypes (v0.x) used Ideal Class Groups (Commutative).
* **Problem:** Commutative groups lacked sufficient expansion properties (slow mixing) and the "arrow of time" was ambiguous ($AB = BA$).
* **Decision:** Adopt Definite Quaternion Algebras ($B_{p, \infty}$).
* **Impact:**
    * **Search Efficiency:** State space is now a Ramanujan Graph (Optimal Expander).
    * **Causality:** Non-commutativity enforces strict path ordering.
    * **Security:** Hardness rests on Supersingular Isogeny problems.

---

## 3. Dynamic Topology

### ADR-004: Spectral Governance & Algebra Migration
* **Problem:** VAPO searches could get trapped in subgraphs with poor connectivity (vanishing spectral gap).
* **Decision:** Implement a **Spectral Governor** that monitors eigenvalue $\lambda_2$. If the gap closes, trigger **Algebra Migration** (change prime $p$).
* **Impact:** The system avoids topological dead ends by shifting the underlying universe.

### ADR-005: State Lifting
* **Problem:** How to preserve knowledge when the universe ($p$) changes?
* **Decision:** Implement a **Lifter** that projects states to a coordinate-free Modular Form feature space, then re-quantizes in the new algebra.
* **Impact:** Enables continuous evolution across discontinuous algebraic structures.
