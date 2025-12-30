# HYPER-TENSOR PROTOCOL (HTP): Core Protocol Specification

## Abstract

This document defines the Hyper-Tensor Protocol (HTP) v1.1. It establishes the Dual-Operator Algebra used to verify the causal integrity (Time) and holographic consistency (Space) of the Evolver system.

**Mathematical Rigor Note:** The underlying algebraic structure is the Ideal Class Group of an imaginary quadratic field, denoted as:

$$Cl(\Delta)$$

where:

$$\Delta < 0$$

is the fundamental discriminant. All group operations (denoted by $\circ$) refer to Gauss Composition of binary quadratic forms, and equality refers to equivalence within the class group.

## 1. The Time Operator: Non-Commutative Evolution

### 1.1 Problem Definition

HTP enforces order sensitivity in the temporal dimension. History cannot be rewritten.

Let $S_t \in Cl(\Delta)$ be the state at step $t$. The state transition is defined as an affine transformation within the group:

$$S_t = \mathcal{F}(S_{t-1}, P_t, h_t) = S_{t-1}^{P_t} \circ G^{h_t}$$

Where:

* $$S_{t-1}^{P_t}$$: The element $S_{t-1}$ raised to the integer power $P_t$ (repeated composition).
* $$\circ$$: The binary operation (composition) in $Cl(\Delta)$.
* $$P_t \in \mathbb{Z}$$: Prime representative of the event/token at step $t$.
* $$h_t \in \mathbb{Z}$$: Hash of the spacetime depth $H(t)$.
* $$G \in Cl(\Delta)$$ : A generator element of the class group (or a subgroup thereof).

### 1.2 Derivation of Time Composition Law ($\oplus_{\text{time}}$)

We define the affine tuple $\mathcal{A} = (P, Q)$ where $P \in \mathbb{Z}$ and $Q \in Cl(\Delta)$. The action $\rho$ of a tuple on a state $S$ is defined as:

$$\rho(\mathcal{A}, S) = S^P \circ Q$$

For two consecutive transformations $\mathcal{A}_1 = (P_1, Q_1)$ and $\mathcal{A}_2 = (P_2, Q_2)$, the merged operator is derived by composing the actions:

$$\begin{aligned}
\rho(\mathcal{A}_2, \rho(\mathcal{A}_1, S)) &= (S^{P_1} \circ Q_1)^{P_2} \circ Q_2 \\
&= (S^{P_1})^{P_2} \circ Q_1^{P_2} \circ Q_2 \quad \text{(by Abelian property of } Cl(\Delta)) \\
&= S^{P_1 P_2} \circ (Q_1^{P_2} \circ Q_2)
\end{aligned}$$

Thus, the Time Operator is defined as:

$$\mathcal{A}_1 \oplus_{\text{time}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1^{P_2} \circ Q_2)$$

### 1.3 Proof of Associativity

For Segment Trees to function, the operator must be associative:

$$(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3 \equiv \mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$$

**Left Side:** $(\mathcal{A}_1 \oplus \mathcal{A}_2) \oplus \mathcal{A}_3$

$$= (P_1 P_2, \quad Q_1^{P_2} \circ Q_2) \oplus (P_3, Q_3)$$

$$= (P_1 P_2 P_3, \quad (Q_1^{P_2} \circ Q_2)^{P_3} \circ Q_3)$$

$$= (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} \circ Q_2^{P_3} \circ Q_3)$$

**Right Side:** $\mathcal{A}_1 \oplus (\mathcal{A}_2 \oplus \mathcal{A}_3)$

$$= (P_1, Q_1) \oplus (P_2 P_3, \quad Q_2^{P_3} \circ Q_3)$$

$$= (P_1 (P_2 P_3), \quad Q_1^{P_2 P_3} \circ (Q_2^{P_3} \circ Q_3))$$

$$= (P_1 P_2 P_3, \quad Q_1^{P_2 P_3} \circ Q_2^{P_3} \circ Q_3)$$

**Conclusion:** The Time Operator is Associative but Non-Commutative (since generally $\mathcal{A}_1 \oplus \mathcal{A}_2 \neq \mathcal{A}_2 \oplus \mathcal{A}_1$).

## 2. The Space Operator: Commutative Aggregation

### 2.1 The Dimensional Requirement

To ensure holographic consistency (the ability to verify from any axis), spatial aggregation must be Commutative.

### 2.2 Derivation of Space Composition Law ($\otimes_{\text{space}}$)

We leverage the intrinsic Abelian property of the Class Group $Cl(\Delta)$ and integer multiplication. We define the Space Operator as component-wise aggregation:

$$\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1 \circ Q_2)$$

Where:

* $$P_1 \cdot P_2$$: Integer multiplication in $\mathbb{Z}$.
* $$Q_1 \circ Q_2$$: Group composition in $Cl(\Delta)$.

### 2.3 Proof of Commutativity

Since $\mathbb{Z}$ (under multiplication) and $Cl(\Delta)$ (under composition) are Abelian:

$$P_1 \cdot P_2 = P_2 \cdot P_1$$

$$Q_1 \circ Q_2 = Q_2 \circ Q_1$$

Therefore:

$$\mathcal{A}_1 \otimes_{\text{space}} \mathcal{A}_2 = \mathcal{A}_2 \otimes_{\text{space}} \mathcal{A}_1$$

## 3. Hyper-Tensor Folding

The Hyper-Tensor $\mathcal{T}$ uses a hybrid topology:

* **Micro-Cells (Time):** Internal neuron history is aggregated via $\oplus_{\text{time}}$.
* **Macro-Grid (Space):** Tensor dimensions are folded via $\otimes_{\text{space}}$.

### 3.1 The Folding Operator $\Phi$

For a tensor of dimension $d$, folding along dimension $k$ uses the Space Operator:

$$\text{Fold}_k(\mathcal{T}) = \bigotimes_{i=1}^{L} \mathcal{T}_{(i, \dots)}$$

### 3.2 Proof of Orthogonal Consistency

We assert that for any two axes $x, y$, the order of folding does not matter:

$$\text{Fold}_y(\text{Fold}_x(\mathcal{T})) \equiv \text{Fold}_x(\text{Fold}_y(\mathcal{T}))$$

**Proof:**
Let $\mathcal{T}_{ij}$ be the element at coordinate $x=i, y=j$.

**LHS:**

$$\text{Fold}_y \left( \bigotimes_i \mathcal{T}_{ij} \right) = \bigotimes_j \left( \bigotimes_i \mathcal{T}_{ij} \right) = \coprod_{j} \coprod_{i} \mathcal{T}_{ij}$$

(Note: $\coprod$ here denotes the iterative application of the Space Operator)

**RHS:**

$$\text{Fold}_x \left( \bigotimes_j \mathcal{T}_{ij} \right) = \bigotimes_i \left( \bigotimes_j \mathcal{T}_{ij} \right) = \coprod_{i} \coprod_{j} \mathcal{T}_{ij}$$

Since the operation $\otimes_{\text{space}}$ is derived from Abelian groups, the terms can be reordered arbitrarily. Thus, LHS $\equiv$ RHS.

Q.E.D.

## 4. Security Assumptions

### 4.1 Time Security (Hidden Order)

Security relies on the infeasibility of computing the Class Number:

$$h(\Delta) = |Cl(\Delta)|$$

An adversary cannot forge a history proof $(W, R)$ such that:

$$W^P \circ R = T$$

(where $=$ denotes equivalence in the group) without solving the discrete logarithm problem or computing the order of the group, which is computationally intractable for large discriminants.

### 4.2 Space Security (Adaptive Root)

Forging a spatial inclusion proof requires solving the root problem:

$$X^e = Y$$

in $Cl(\Delta)$. Under the Strong RSA Assumption (adapted to Class Groups), finding $e$-th roots is hard when the order of the group is unknown.

### 4.3 The Kernel Trap (Boundary Analysis)

**Mathematical Possibility:**
While $\oplus_{\text{time}}$ generally ensures perturbation propagation, a "Kernel Trap" exists if the perturbation $\varepsilon \in Cl(\Delta)$ (where $\varepsilon \neq 1_{Cl(\Delta)}$) falls into the kernel of the power map $x \mapsto x^P$:

$$\varepsilon^P = 1_{Cl(\Delta)}$$

This implies that the order of the element, $\text{ord}(\varepsilon)$, must divide $P$.

**Engineering Mitigation:**

* **Huge Class Number:** The discriminant size ($\ge 2048$ bits) implies $h(\Delta) \approx \sqrt{|\Delta|} \approx 2^{1024}$. The probability of randomly encountering an element with small order is negligible.
* **Large Primes:** $P$ is chosen as a large prime (e.g., 64-bit). For $\text{ord}(\varepsilon) \mid P$, since $P$ is prime, the order must be exactly $P$. Finding an element of specific order $P$ without knowing the factorization of the group order $h(\Delta)$ is computationally infeasible.

## 5. The Bias-Control Interface (Effective Model)

For the historical derivation of exact controllability, see `Security vs. Trainability.md`.

### 5.1 The Control Problem

The HTP Protocol requires the system to output a specific Algebraic Root $Q_{target} \in Cl(\Delta)$ that satisfies logical constraints ($Energy = 0$). However, the Generator outputs a probabilistic vector (Logits).

We define the Effective Control Model as a function that maps a control signal $\vec{b}$ to a Token selection.

### 5.2 The Unified Bias Definition

The Bias Vector $\vec{b}$ operates across three layers of abstraction:

* **Layer 0 (Control State):**
    $$\vec{b}_{ctrl} \in (\mathbb{Z}/L\mathbb{Z})^{16}$$
    The discrete object optimized by VAPO.

* **Layer 1 (Embedding):**
    $$\phi(\vec{b}_{ctrl}) \in \mathbb{R}^{32}$$
    A continuous cyclic embedding preserving topology.

* **Layer 2 (Projection):**
    $$\vec{b}_{logits} = W_{proj} \cdot \phi(\vec{b}_{ctrl})$$
    The force applied to the neural manifold.

### 5.3 Protocol Binding

A valid HTP Proof Bundle must commit to both the Algebra and the Control Signal:

$$\text{ProofBundle} := \{ \text{GlobalRoot}_{\text{alg}}, \vec{b}_{ctrl}, \text{Proof}_{\text{validity}} \}$$

The verification logic holds if and only if:

$$\text{Action} = \text{Argmax}( \text{Logits}_{gen} + \text{Project}(\vec{b}_{ctrl}) )$$

This ensures that the "correction" applied to the logic is explicitly revealed and cryptographically bound to the context.
