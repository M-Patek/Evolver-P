# Security vs. Trainability: The Engineering Cheat

"We replaced the perfect math with a guided missile."

## 1. The Core Paradox

In the design of Neuro-Symbolic systems, we encounter a fundamental trade-off between Security (Cryptography) and Trainability (Gradient Descent).

### The Security Demand: Chaos

**Requirement:** To prevent history rewriting, operations must be Non-Commutative and Chaotic.

**Math:** $$x(t+1) = x(t)^P \cdot Q$$

Small changes in input must cause avalanche changes in output (Butterfly Effect).

**Result:** The function surface is fractal and jagged. Gradient $\nabla E$ is undefined or infinite.

### The Trainability Demand: Smoothness

**Requirement:** To learn patterns via Backpropagation, operations must be Lipschitz Continuous.

**Math:** $$|f(x) - f(y)| \le K |x - y|$$

**Result:** The function surface must be smooth valleys.

**Conclusion:** You cannot backpropagate through a cryptographic hash function. A system cannot be both perfectly secure (Chaotic) and perfectly trainable (Smooth) at the same core layer.

---

## 2. The Solution: "The Sidecar Architecture"

Instead of trying to make the cryptographic core differentiable (impossible), we split the system into two distinct parallel tracks:

### Track A: The Generator (The Smooth Brain)
* **Type:** Standard Transformer / Neural Network.
* **Domain:** Continuous $\mathbb{R}^N$.
* **Role:** Intuition, pattern matching, hallucination.
* **Trainability:** 100%.

### Track B: The HTP Core (The Jagged Rail)
* **Type:** Algebraic State Machine (STP).
* **Domain:** Discrete Groups $Cl(\Delta)$.
* **Role:** Verification, constraint checking, history binding.
* **Trainability:** 0% (Fixed Rules).

### The Bridge: VAPO (Valuation-Adaptive Perturbation Optimization)
We introduce a Bias Channel that allows Track B to steer Track A. We do not train Track A to be perfect; we assume it is flawed and use Track B to apply runtime corrections.

---

## Appendix A: The Theoretical Ancestor (Project "Native Token")

The following section describes the original "Mark-I" design of Evolver. While the current implementation uses the Sidecar approach, this math proves that a solution vector $\vec{b}$ ALWAYS exists.

In the original design, we hypothesized a system where the output token was directly determined by an affine shift on a torus.

### A.1 Definitions (The Idealized Model)

**Definition (Coordinate Space as a Torus)**
Let the decoder coordinate space be the finite Abelian group:

$$\text{Coord} := (\mathbb{Z}/L\mathbb{Z})^d$$

Define the per-dimension circular distance:

$$d_L(x, y) := \min(|x - y|, L - |x - y|)$$

**Definition (Native Bias Channel)**
Let $\vec{b} \in \text{Coord}$ be a bias vector. The output was defined strictly as:

$$\text{OutCoord} := (\Psi(Q_{\text{alg}}) + \vec{b}) \pmod L$$

### A.2 Lemma: Stability of the Linear Bias Channel

**Lemma (Translation Isometry; 1-Lipschitz Stability)**
For fixed $Q_{\text{alg}}$, the mapping $T(\vec{b}) := \text{OutCoord}(Q_{\text{alg}}, \vec{b})$ is an isometry on $(\text{Coord}, d)$.

**Proof:**

$$T(\vec{b} + \delta) - T(\vec{b}) = (\Psi(Q) + \vec{b} + \delta) - (\Psi(Q) + \vec{b}) = \delta \pmod L$$

Component-wise, translation does not change circular differences, hence distances are preserved:

$$d(T(\vec{b} + \delta), T(\vec{b})) = d(\delta, \vec{0})$$

This implies the control surface is perfectly smooth (1-Lipschitz) in the ideal model, justifying the use of gradient-free optimization in the Sidecar model.

### A.3 Theorem: Exact Controllability (Existence Proof)

**Theorem (Surjectivity)**
Fix the chaotic algebraic state $Q$ and define $\vec{c} := \Psi(Q)$. For any target logical token $T$ located at coordinate $\vec{c}^*$, there exists a unique bias vector $\vec{b}^* \in \text{Coord}$ such that:

$$\text{OutCoord}(Q, \vec{b}^*) = \vec{c}^*$$

**Proof:**
We need to solve for $\vec{b}^*$:

$$(\vec{c} + \vec{b}^*) \pmod L = \vec{c}^*$$

Since addition in $\text{Coord}$ (a Torus) is a group action by translations, it is bijective. The unique solution is:

$$\vec{b}^* := \vec{c}^* - \vec{c} \pmod L$$

**Q.E.D.**
This theorem provides the Existence Guarantee for the VAPO algorithm: a solution always exists, we just need to search for it.

### A.4 Theorem: Verifiability (No Splicing)

**Theorem (Proof-Carrying Bias)**
Assume:
1. $\text{Proof}_{alg}$ verifies $Q$ against $\text{GlobalRoot}_{alg}$.
2. $\text{Proof}_{bias}$ verifies $\vec{b}$ against $\text{GlobalRoot}_{bias}$.
3. Both roots are bound to a shared context ctx.

Then any verifier can deterministically recompute $\text{OutCoord} = (\Psi(Q) + \vec{b}) \pmod L$. The server cannot "splice" a $Q$ from one run and a $\vec{b}$ from another to forge a result, because the ctx binding would fail verification in at least one proof.

**Q.E.D.**
