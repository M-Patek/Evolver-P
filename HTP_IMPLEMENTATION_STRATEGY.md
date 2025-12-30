# HTP Implementation Strategy: Decoupling Logic from Cryptography

**Core Philosophy:** "The Control Law is invariant under the choice of the underlying Group."

This document clarifies why the Evolver system can rigorously validate its Neuro-Symbolic alignment logic (Phase 1) without immediately implementing the full cryptographic primitives of the Hyper-Tensor Protocol (Phase 2).

---

## 1. The Abstraction of Chaos

The Evolver architecture is designed with a strict separation of concerns:
* **The Controller (VAPO):** Observes energy gradients and outputs bias vectors.
* **The Plant (HTP Core):** Accepts actions and evolves the state.

### Why the Specific Group Doesn't Matter (Yet)
The Bias Controller relies on **Theorem 5.7 (Controllability)**. This theorem states that for any target state in a coordinate space, there exists a bias vector $\vec{b}$ that can map the chaotic output to it.

Mathematically, the Controller treats the HTP Core as a "Black Box" function $F(x)$:

$$State_{next} = F(State_{current}, Action + Bias)$$

For the purpose of validating the VAPO Algorithm, we only need $F(x)$ to satisfy two properties:
1.  **Determinism:** Same input $\rightarrow$ Same output.
2.  **Algebraic Consistency:** It follows Group Axioms (Associativity, Identity, Invertibility).

Whether $F(x)$ is implemented using Class Groups ($Cl(\Delta)$) or Simple Matrix Multiplication ($GL(n, \mathbb{Z})$), the dynamics of the control loop remain identical. If VAPO can stabilize a simple matrix system, it can mathematically stabilize a Class Group system.

---

## 2. Phase 1: Logic Verification (Current Status)

**Goal:** Prove that a chaotic generator can be forced to follow strict logical rules via external bias.

In this phase, we use a **"Mock Chaos"** backend:
* **Time Operator:** $A * B$ (Matrix Multiplication or Linear Congruential Generator).
* **Space Operator:** $A + B$ (Vector Addition).
* **Commitment:** DefaultHasher (Rust std lib).

**Justification:**
* **Speed:** Matrix operations are $\approx 1000x$ faster than Class Group composition, allowing rapid iteration of the VAPO hyperparameters.
* **Debuggability:** We can easily reverse-engineer a matrix state to see why a specific logic failed, which is impossible with cryptographic one-way functions.
* **Sufficient Complexity:** A random matrix group is already sufficiently "chaotic" to test if the Bias Controller can overcome non-linear trajectories.

---

## 3. Phase 2: Cryptographic Hardness (Target)

**Goal:** Ensure the history cannot be rewritten (Time Security) and proofs can be folded (Space Holography).

In this phase, we swap the backend for **Class Groups of Imaginary Quadratic Fields**:
* **Time Operator:** Composition of quadratic forms $(a, b, c)$.
* **Hidden Order:** We rely on the difficulty of finding the order of the group to prevent "time travel" (computing inverse operations efficiently).
* **VDF Properties:** Verifiable Delay Functions ensure the proof generation required actual sequential work.

### The "Drop-In" Replacement
Because the entire system communicates via the **HTP Interface** (defined in `src/dsl/stp_bridge.rs`), switching from Mock Algebra to Class Group Algebra is a software engineering task, not a research task.

```rust
// Current (Phase 1)
type AlgebraicState = Matrix<f64>; 

// Future (Phase 2)
type AlgebraicState = ClassGroupElement<Discriminant>; 
```

The Bias Controller does not need to change a single line of code, as it operates on the **Energy Surface** produced by the state, not the state itself.

---

## 4. Conclusion

We are currently validating the **Control Theory** aspect of Evolver. The Cryptographic Security is a modular component that will be plugged in once the control loop is proven stable.

**Summary:** We use "Simulated Chaos" to train the Pilot (Controller). Once the Pilot is ready, we will put them in the real F-22 Raptor (Class Groups).
