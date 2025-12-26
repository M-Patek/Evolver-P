# HYPER-TENSOR PROTOCOL (HTP): Theoretical Proofs

## Abstract

This document provides the formal mathematical derivations for the HYPER-TENSOR PROTOCOL (HTP). It establishes the Dual-Operator Architecture: using non-commutative affine evolution for temporal integrity, and commutative Abelian aggregation for spatial holography. This separation guarantees both historical order sensitivity and multi-dimensional verification consistency.

## 1. The Time Operator: Non-Commutative Evolution

### 1.1 Problem Definition

In standard accumulators, operations are commutative ($x^{ab} = x^{ba}$), allowing history rewriting. HTP enforces order sensitivity in the temporal dimension.

Let $S_t$ be the state at step $t$. The state transition is defined as:

$$
S_t = \mathcal{F}(S_{t-1}, P_t, h_t) = S_{t-1}^{P_t} \cdot G^{h_t} \pmod \Delta
$$

Where:
* $P_t$: Prime representative of the event/token at step $t$.
* $h_t$: Hash of the spacetime depth $H(t)$.
* $G$: Generator of the class group.

### 1.2 Recursive Expansion

We express state $S_n$ as a function of previous state $S_{k-1}$:

$$
S_n = S_{k-1}^{\left( \prod_{i=k}^n P_i \right)} \cdot \left( G^{h_k \cdot \prod_{j=k+1}^n P_j} \cdot \dots \cdot G^{h_n} \right)
$$

This structure proves that any change in the sequence $P_k \dots P_n$ fundamentally alters the final state $S_n$.

### 1.3 Derivation of Time Composition Law ($\oplus_{\text{time}}$)

To enable efficient verification, we define the affine tuple $\mathcal{A} = (P, Q)$ acting on state $S$ as $\rho(\mathcal{A}, S) = S^P \cdot Q$. For two consecutive transformations $\mathcal{A}_1 = (P_1, Q_1)$ and $\mathcal{A}_2 = (P_2, Q_2)$, the merged operator is derived as:

$$
\begin{aligned}
\rho(\mathcal{A}_2, \rho(\mathcal{A}_1, S)) &= (S^{P_1} \cdot Q_1)^{P_2} \cdot Q_2 \\
&= S^{P_1 P_2} \cdot (Q_1^{P_2} \cdot Q_2)
\end{aligned}
$$

Thus, the Time Operator is defined as:

$$
\mathcal{A}_1 \oplus_{\text{time}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1^{P_2} \cdot Q_2)
$$

### 1.4 Associativity Proof

For Segment Trees to function, the operator must be associative: $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3 \equiv \mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$.

**Left Side:** $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3$

$$
= (P_1 P_2 P_3, \quad (Q_1^{P_2} Q_2)^{P_3} Q_3) = (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3)
$$

**Right Side:** $\mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$

$$
= (P_1 (P_2 P_3), \quad Q_1^{P_2 P_3} (Q_2^{P_3} Q_3)) = (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} Q_2^{P_3} Q_3)
$$

**Conclusion:** The Time Operator is Associative but Non-Commutative.

## 2. The Space Operator: Commutative Aggregation

### 2.1 The Dimensional Conflict

Previous attempts to use $\oplus_{\text{time}}$ for spatial folding failed because non-commutativity implies $\text{Fold}_y(\text{Fold}_x(\mathcal{T})) \neq \text{Fold}_x(\text{Fold}_y(\mathcal{T}))$, making orthogonal verification impossible.

### 2.2 Derivation of Space Composition Law ($\otimes_{\text{space}}$)

To ensure holographic consistency, spatial aggregation must be Commutative. We leverage the intrinsic Abelian property of the Class Group $Cl(\Delta)$ and integer multiplication. We define the Space Operator as component-wise aggregation:

$$
\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1 \cdot Q_2)
$$

Where $Q_1 \cdot Q_2$ is standard group multiplication.

### 2.3 Proof of Commutativity

Since $\mathbb{Z}$ and $Cl(\Delta)$ are Abelian:
$P_1 \cdot P_2 = P_2 \cdot P_1$
$Q_1 \cdot Q_2 = Q_2 \cdot Q_1$

Therefore:

$$
\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = \mathcal{A}_2 \otimes_{\text{space}} \mathcal{A}_1
$$

## 3. Hyper-Tensor Folding & Verification

### 3.1 Tensor Structure

The Hyper-Tensor $\mathcal{T}$ uses a hybrid topology:
* **Micro-Cells (Time):** Internal neuron history is aggregated via $\oplus_{\text{time}}$.
* **Macro-Grid (Space):** Tensor dimensions are folded via $\otimes_{\text{space}}$.

### 3.2 The Folding Operator $\Phi$

For a tensor of dimension $d$, folding along dimension $k$ uses the Space Operator:

$$
\text{Fold}_k(\mathcal{T}) = \bigotimes_{i=1}^{L} \mathcal{T}_{(i, \dots)}
$$

### 3.3 Orthogonal Consistency Proof

We assert that for any two axes $x, y$:

$$
\text{Fold}_y(\text{Fold}_x(\mathcal{T})) \equiv \text{Fold}_x(\text{Fold}_y(\mathcal{T}))
$$

**Proof:**
Let $\mathcal{T}_{ij}$ be the element at $x=i, y=j$.

**LHS:** $\bigotimes_j (\bigotimes_i \mathcal{T}_{ij}) = \prod_{j} \prod_{i} \mathcal{T}_{ij}$ (Product notation for Abelian group op)

**RHS:** $\bigotimes_i (\bigotimes_j \mathcal{T}_{ij}) = \prod_{i} \prod_{j} \mathcal{T}_{ij}$

Since the product is over a finite Abelian group, the order of terms does not matter.
**Q.E.D.**

## 4. Security Reductions

### 4.1 Time Security (Hidden Order Assumption)

The security of the time dimension relies on the infeasibility of finding the order of $Cl(\Delta)$. An adversary cannot forge a history proof $(W, R)$ such that $W^P \cdot R \equiv T$ without solving the discrete log or order problem.

### 4.2 Space Security (Strong RSA / Adaptive Root)

The security of the space dimension, effectively a product of primes and group elements, relies on the Strong RSA assumption (for $P$ factor factorization) and the Adaptive Root Assumption in Class Groups (for $Q$ aggregation). Forging a spatial inclusion proof requires solving the root problem $X^e \equiv Y \pmod \Delta$.

### 4.3 The Kernel Trap (Boundary Analysis)

#### 4.3.1 Mathematical Possibility
While the Non-Commutative Time Operator ($\oplus_{\text{time}}$) generally ensures that any perturbation $\varepsilon$ in the input state propagates to the output, there exists a theoretically possible boundary condition known as **"The Kernel Trap."**

Let the perturbation be $\varepsilon \neq 1$.
If $\varepsilon$ falls into the kernel of the power map $x \mapsto x^P$, i.e.,
$$
\varepsilon^P \equiv 1 \pmod \Delta
$$
then the output state remains unchanged despite the input mutation:
$$
\rho(\mathcal{A}, S \cdot \varepsilon) = (S \cdot \varepsilon)^P \cdot Q = S^P \cdot \varepsilon^P \cdot Q = S^P \cdot 1 \cdot Q = \rho(\mathcal{A}, S)
$$

Mathematically, this occurs if and only if the order of the perturbation element, denoted as $\text{ord}(\varepsilon)$, divides the semantic prime $P$:
$$
\text{ord}(\varepsilon) \mid P
$$

#### 4.3.2 Engineering Mitigation
In the HTP engineering implementation, we render the probability of falling into the Kernel Trap negligible through three layers of defense:

1.  **Huge Class Number ($h(\Delta)$):**
    By enforcing a discriminant size of $\geq 2048$ bits (see `param.rs`), the size of the Class Group is astronomically large ($\approx \sqrt{|\Delta|}$). This makes the probability of randomly encountering an element with a specific small order effectively zero ($< 2^{-100}$).

2.  **Large Semantic Primes ($P$):**
    The system weights $P$ are generated via `hash_to_prime` and are guaranteed to be large primes (e.g., 64-bit or 128-bit).
    Since $P$ is prime:
    * For $\text{ord}(\varepsilon) \mid P$ to hold, $\text{ord}(\varepsilon)$ must be equal to $P$ (since $\varepsilon \neq 1$).
    * This implies the attacker must find an element $\varepsilon$ whose order is exactly the large prime $P$.

3.  **Small Order Filtering (Code Level):**
    In `algebra.rs`, the `ClassGroupElement::generator` and validation logic explicitly filter out elements with small orders (e.g., 2, 3, 5).

While this does not strictly eliminate elements of order $P$, combined with the **Hidden Order Assumption**, finding an element of a specific large order $P$ without knowing the class number $h(\Delta)$ is computationally equivalent to solving the Discrete Logarithm Problem or factoring the class number, which is infeasible.

**Conclusion:** While the Kernel Trap is a valid algebraic boundary, it is cryptographically inaccessible in the Evolver architecture.

## 5. Bias-Controlled Decoding (Stable Control over a Chaotic Core)

### 5.1 Definitions

**Definition 5.1 (Affine Roots and GlobalRoot_alg)**
Let each micro-cell aggregate its local history using the Time Operator $\oplus_{\text{time}}$, producing an affine tuple $A_{\text{cell}} = (P_{\text{cell}}, Q_{\text{cell}})$.
Let the macro-grid fold all cells using the Space Operator $\otimes_{\text{space}}$, producing the algebraic global root:

$$
A_{\text{alg}} = \bigotimes_{\vec{v} \in T} A_{\text{cell}}(\vec{v}) \quad \text{and} \quad \text{GlobalRoot}_{\text{alg}} := A_{\text{alg}}.
$$

(As established in Sections 1–3, $\oplus_{\text{time}}$ is associative and order-sensitive; $\otimes_{\text{space}}$ is commutative and axis-independent.)

**Definition 5.2 (Coordinate Space as a Torus)**
Let the decoder coordinate space be the finite Abelian group (a discrete torus):

$$
\text{Coord} := (\mathbb{Z}/L\mathbb{Z})^d,
$$

with component-wise addition modulo $L$.
Define the per-dimension circular distance:

$$
d_L(x, y) := \min(|x - y|, L - |x - y|),
$$

and the torus distance for vectors (any norm over per-dimension distances is acceptable; e.g. $\ell_1$):

$$
d(\vec{x}, \vec{y}) := \left\| (d_L(x_1, y_1), \dots, d_L(x_d, y_d)) \right\|.
$$

**Definition 5.3 (Chaotic Projection $\Psi$)**
Let $\Psi$ be the (possibly chaotic / one-way) projection used by the decoder:

$$
\Psi: Cl(\Delta) \to \text{Coord}.
$$

No continuity, smoothness, or Lipschitz assumptions are made about $\Psi$. In particular, $\Psi$ may be induced by reduction/canonicalization and thus be effectively discontinuous.

**Definition 5.4 (Linear Bias Channel and Final Output Coordinate)**
Let $\vec{b} \in \text{Coord}$ be a bias vector.
Define the bias-augmented output coordinate as:

$$
\text{OutCoord}(A_{\text{alg}}, \vec{b}) := (\Psi(Q_{\text{alg}}) + \vec{b}) \pmod L,
$$

where $A_{\text{alg}} = (P_{\text{alg}}, Q_{\text{alg}})$.

*Interpretation:*
* **Channel A (logic):** $\Psi(Q_{\text{alg}})$ remains chaotic and provides jump-like addressing.
* **Channel B (control):** $\vec{b}$ is a linear translation in $\text{Coord}$, providing stable and predictable micro-adjustments.

**Definition 5.5 (Bias Commitment and ProofBundle)**
Let the bias field over tensor coordinates be $\{\vec{b}(\vec{v})\}_{\vec{v} \in T}$, committed via an axis-independent space construction (e.g., a bias tensor folded by $\otimes_{\text{space}}$, or a Merkle commitment).
Denote the commitment root as $\text{GlobalRoot}_{\text{bias}}$.

Define a composite root that binds algebra, bias, and context:

$$
\text{GlobalRoot} := H(\text{Ser}(\text{GlobalRoot}_{\text{alg}}) \parallel \text{Ser}(\text{GlobalRoot}_{\text{bias}}) \parallel \text{ctx}),
$$

where $\text{ctx}$ includes request-scoped identifiers (e.g., request_id, log_epoch, model_id, etc.), $\text{Ser}(\cdot)$ is a canonical serialization, and $H$ is a cryptographic hash.

A **ProofBundle** is defined as:

$$
\text{ProofBundle} := (\text{ctx}, \text{GlobalRoot}_{\text{alg}}, \text{GlobalRoot}_{\text{bias}}, \text{GlobalRoot}, \text{Proof}_{\text{alg}}, \text{Proof}_{\text{bias}})
$$

where:
* $\text{Proof}_{\text{alg}}$ proves the claimed $Q_{\text{alg}}$ (or the relevant cell root feeding into it) is consistent with $\text{GlobalRoot}_{\text{alg}}$, using $\oplus_{\text{time}}$ (micro) and $\otimes_{\text{space}}$ (macro) inclusion/aggregation.
* $\text{Proof}_{\text{bias}}$ proves the bias vector $\vec{b}$ used for output is the committed value under $\text{GlobalRoot}_{\text{bias}}$ for the same $\text{ctx}$.

### 5.2 Lemma: Stability of the Linear Bias Channel

**Lemma 5.6 (Translation Isometry; 1-Lipschitz Stability)**
For fixed $A_{\text{alg}}$, the mapping

$$
T(\vec{b}) := \text{OutCoord}(A_{\text{alg}}, \vec{b})
$$

is an isometry on $(\text{Coord}, d)$.
In particular, for any $\delta \in \text{Coord}$:

$$
d(T(\vec{b} + \delta), T(\vec{b})) = d(\delta, \vec{0}).
$$

**Proof.**

$$
T(\vec{b} + \delta) - T(\vec{b}) = (\Psi(Q_{\text{alg}}) + \vec{b} + \delta) - (\Psi(Q_{\text{alg}}) + \vec{b}) = \delta \pmod L
$$

Component-wise, translation does not change circular differences, hence distances are preserved by definition of $d_L$.
**Q.E.D.**

### 5.3 Theorem: Exact Controllability in Coordinate Space

**Theorem 5.7 (Surjectivity / Exact Control by $\vec{b}$)**
Fix $A_{\text{alg}}$ and define $\vec{c} := \Psi(Q_{\text{alg}})$.
For any target coordinate $\vec{c}^* \in \text{Coord}$, there exists a unique $\vec{b}^* \in \text{Coord}$ such that

$$
\text{OutCoord}(A_{\text{alg}}, \vec{b}^*) = \vec{c}^*,
$$

namely

$$
\vec{b}^* := \vec{c}^* - \vec{c} \pmod L.
$$

**Proof.**
Direct substitution yields:

$$
(\vec{c} + \vec{b}^*) \pmod L = (\vec{c} + (\vec{c}^* - \vec{c})) \pmod L = \vec{c}^*.
$$

Uniqueness follows because addition in $\text{Coord}$ is a group action by translations, hence bijective.
**Q.E.D.**

**Corollary 5.8 (Finite Convergence Under Discrete LocalShift)**
Assume LocalShift is restricted to unit moves $\delta \in \{\pm \vec{e}_i\}_{i=1}^d$ (one coordinate changes by $\pm 1 \pmod L$ per step).
Then from any $\vec{b}$, there exists a deterministic sequence of LocalShift moves reaching $\vec{b}^*$ in exactly $\sum_{i=1}^d d_L(b_i, b^*_{,i})$ steps.
Therefore, reachability/convergence in the $\vec{b}$-space is guaranteed without any assumption on the continuity of $\Psi$.

### 5.4 Theorem: Verifiability of Bias-Augmented Outputs

**Theorem 5.9 (Proof-Carrying Bias; No Splicing Under Binding)**
Assume:
1.  $\text{Proof}_{\text{alg}}$ verifies $Q_{\text{alg}}$ against $\text{GlobalRoot}_{\text{alg}}$ under the same $\text{ctx}$.
2.  $\text{Proof}_{\text{bias}}$ verifies the revealed $\vec{b}$ against $\text{GlobalRoot}_{\text{bias}}$ under the same $\text{ctx}$.
3.  $\text{GlobalRoot}$ binds $\text{GlobalRoot}_{\text{alg}}$, $\text{GlobalRoot}_{\text{bias}}$, and $\text{ctx}$ as in Definition 5.5 (or equivalently, $\text{GlobalRoot}_{\text{bias}}$ is included in the Fiat–Shamir transcript for $\text{Proof}_{\text{alg}}$).

Then any verifier can deterministically recompute

$$
\text{OutCoord} = (\Psi(Q_{\text{alg}}) + \vec{b}) \pmod L
$$

and the server cannot change $\text{OutCoord}$ (by swapping either $Q_{\text{alg}}$ or $\vec{b}$) without causing at least one verification step to fail.

**Proof (sketch).**
From (1), $Q_{\text{alg}}$ is bound to $\text{GlobalRoot}_{\text{alg}}$. From (2), $\vec{b}$ is bound to $\text{GlobalRoot}_{\text{bias}}$.
From (3), both roots are cryptographically bound to the same $\text{ctx}$ and to $\text{GlobalRoot}$, preventing mix-and-match (“splicing”) of a valid $\text{Proof}_{\text{alg}}$ from one context with a valid $\text{Proof}_{\text{bias}}$ from another.
Since $\text{OutCoord}$ is a deterministic function of the verified pair $(Q_{\text{alg}}, \vec{b})$, any deviation changes the committed data and breaks verification.
**Q.E.D.**

### 5.5 Counterexamples and Boundaries (Falsifiability)

**Boundary A (Incorrect Metric Breaks “Continuity” at Wrap-Around)**
If the verifier uses representative integer subtraction on $\{0, \dots, L-1\}$ instead of the torus distance $d_L$, the stability claim fails at wrap-around.
*Example:* $L=32, b=31, \delta=1$ yields $(b+\delta) \pmod L = 0$ and naive $|0 - 31| = 31 \neq 1$.
Therefore Lemma 5.6 requires a circular/torus metric.

**Boundary B (Splicing Attack Without Binding)**
If $\text{Proof}_{\text{alg}}$ and $\text{Proof}_{\text{bias}}$ are verified independently without a shared $\text{ctx}$ binding (or without $\text{GlobalRoot}$ composition), an adversary can combine:
* a valid $\text{Proof}_{\text{alg}}$ from run A, and
* a valid $\text{Proof}_{\text{bias}}$ from run B,
passing both checks while producing an output coordinate inconsistent with either run’s intended decoding.
Thus Theorem 5.9 requires explicit binding.

**Boundary C (Constrained Bias Space Reduces Reachability)**
If $\vec{b}$ is restricted to a subset (e.g., bounded norm, sparsity, fixed Hamming weight, limited update budget), Theorem 5.7 (exact surjectivity) no longer holds.
Only reachability within that subset can be claimed, and convergence guarantees must be restated under the constraint model.

**Boundary D (Coordinate Verifiability $\neq$ Token Verifiability)**
Theorem 5.9 guarantees correctness of the verified coordinate $\text{OutCoord}$.
If token selection uses an unproven heuristic (e.g., approximate KNN) the server may still claim an arbitrary token not consistent with $\text{OutCoord}$.
Full token-level trust requires a verifiable mapping (e.g., committed vocabulary coordinates plus a verifiable nearest-neighbor proof, or a deterministic decode rule whose steps are provable).
