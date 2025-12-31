# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

"Time is Evolution, Space is Projection."

## 1. Mathematical Preliminaries

### 1.1 Class Group Parameters & The Unknown Order

**Discriminant Generation:**
Define the discriminant $\Delta$ as a hash-derived parameter:

$$\Delta = -M$$

where $M$ is a large prime number (e.g., 2048-bit) derived deterministically from the ContextHash satisfying $M \equiv 3 \pmod 4$.

**The Open Secret:**
While $\Delta$ is public, the group structure of $Cl(\Delta)$ remains opaque. Specifically, the Class Number $h(\Delta)$ (the order of the group) is computationally infeasible to calculate for large discriminants (Cohen-Lenstra Heuristics).

**Security Consequence:**
The hardness of computing $h(\Delta)$ ensures that the group acts as a Group of Unknown Order. This prevents attackers (and the generator) from using Lagrange's Theorem ($g^{|G|} = 1$) to compute shortcuts for exponentiation.

### 1.2 Algebraic Dynamics (The Arrow of Time)

We distinguish between the Commutative State Space and the Non-Commutative Time Evolution.

**State Space (Space):**
The Ideal Class Group $Cl(\Delta)$ is an Abelian group.

$$S_1 \circ S_2 = S_2 \circ S_1$$

**Time Operator (VDF Action):**
Time evolution is defined by the Squaring Operation, which acts as a Verifiable Delay Function (VDF) in an unknown order group.

$$\Phi(S) = S^2$$

**Evolution Step:**
A single step of evolution combines the deterministic passage of time (Squaring) with the exertion of Will (Perturbation $\epsilon$).

$$S_{t+1} = \Phi(S_t) \circ \epsilon_t = S_t^2 \circ \epsilon_t$$

**Irreversibility:**
Computing square roots in a class group of unknown order is computationally equivalent to factoring the discriminant, which is hard. This ensures the evolution cannot be easily reversed.

---

## 2. Affine Evolution & Optimization (The Soul & Will)

### 2.1 The Algebraic State (Soul)

The state $S \in Cl(\Delta)$ is defined as an equivalence class of binary quadratic forms $[a, b, c]$.

### 2.2 The Will's Loop (Search)

Implemented in `src/will/optimizer.rs`.

* **Input:** Current state $S_t$.
* **Action:** The Will selects a perturbation $\epsilon \in \mathcal{P}$ (Generator Set) to navigate the Cayley Graph.
* **Dynamics:** The optimizer searches for a path of perturbations such that the resulting state minimizes the STP energy.
* **Constraint:** The search is a discrete optimization process. There are no gradients; the "Will" must physically traverse the graph nodes to evaluate them.

### 2.3 Materialization (The Body's Projection)

Implemented in `src/body/decoder.rs`.

* **Input:** The optimized state $S^*$.
* **Process:** Recursive application of the Affine Projection Map $\Psi$.
* **Output:** A sequence of logical action IDs $d_1, d_2, \dots, d_k$.

---

## 3. Hyper-Tensor Topology (v-PuNN)

### 3.1 Coordinate Mapping

Define a mapping $\Psi$ from the algebraic state $S$ to a logical path $\mathcal{P}$ using Linear Congruence Projection:

$$\Psi_k(S) = (a + k \cdot b) \pmod{P}$$

This projection preserves the Lipschitz continuity of the algebraic manifold, allowing local search algorithms (like VAPO) to function effectively.

### 3.2 Orthogonal Verification

The HTP protocol requires that generated logic must be consistent. The STP engine verifies the logical validity of the materialized path.

$$E_{STP}(\text{Materialize}(S^*)) == 0$$

---

## 4. Proof of Will & Security Model

### 4.1 The Security Goal: Unforgeability

The goal of HTP security is not to hide the logic, but to prove that the logic was generated through computational effort (Search) rather than hallucinated or pre-computed via shortcuts.

### 4.2 Proof Bundle (The Artifact)

The output of the system is a verifiable artifact:

$$\text{ProofBundle} := \{ \mathbf{H}_{ctx}, S_{final}, \text{Trace}_{\epsilon} \}$$

* $\mathbf{H}_{ctx}$: Context Hash (anchors $\Delta$).
* $S_{final}$: The result state.
* $\text{Trace}_{\epsilon}$: The sequence of perturbations $[\epsilon_1, \epsilon_2, \dots, \epsilon_k]$ applied.

### 4.3 Verification (Replay)

The Verifier performs the following steps:

1.  **Derive:** Recompute $\Delta$ from $\mathbf{H}_{ctx}$.
2.  **Replay:** Starting from Identity, applying the sequence $S_{t+1} = S_t^2 \circ \epsilon_t$.
3.  **Check:** Verify that the resulting $S_{final}$ yields $E_{STP} == 0$.

### 4.4 Why this is "Secure"? (The Hardness Assumptions)

* **Sequentiality (No Parallelism):**
    Due to the squaring operation $S \to S^2$ in a group of unknown order, computing the final state $S_{final}$ requires $\mathcal{O}(T)$ sequential group operations. It cannot be parallelized.
* **No Shortcuts (No Forgery):**
    Without knowing $h(\Delta)$, an attacker cannot compute $S^{2^T}$ directly. Therefore, they cannot "jump" to a valid solution state $S^*$ without actually traversing the path.
* **Proof of Will (Proof of Search):**
    Finding a trace that results in Zero Energy requires solving a discrete search problem on the graph. The existence of a valid bundle proves that the generator performed the necessary Work (Search) to align the algebra with logic.

---

## 5. Conclusion

The HTP v1.4 specification defines a Proof-of-Will protocol:

* **Algebra:** Unknown Order Class Groups ($Cl(\Delta)$).
* **Dynamics:** Sequential VDF (Squaring).
* **Security:** Computational Unforgeability via Sequentiality.
* **Truth:** The logical state reached only through the exertion of computational Will.
