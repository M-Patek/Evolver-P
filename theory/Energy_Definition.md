# THEORY PATCH: The Structure of STP Energy

## Defining the Evaluation Signal Functional $E$

### 1. The Definition Gap

The symbol $E$ (Energy) serves two conflicting masters in the current architecture:

* **Logical Validity:** A binary judgment (Valid/Invalid).
* **VAPO Guidance:** A continuous gradient-like signal for optimization.

To formalize this, we define Energy not as a simple scalar, but as a **Relaxed Lyapunov Function**.

### 2. Formal Definition

Let $\mathcal{M}$ be the STP algebraic state manifold.
Let $\Phi: \mathcal{M} \times A \to \mathcal{M}$ be the transition dynamics.
Let $s_{target}$ be the target state (or subspace) imposed by a logical assertion.

The **Hard Logical Energy** $E_{logic}$ is defined as:

$$E_{logic}(s, a) = \begin{cases} 0 & \text{if } \Phi(s, a) \subseteq s_{target} \\ \infty & \text{otherwise} \end{cases}$$

The **Soft Optimization Energy** $E_{opt}$ (used by VAPO) is defined as a metric distance on the probability simplex of the state space:

$$E_{opt}(s, a) = || \Phi(s, a) - Proj_{target}(\Phi(s, a)) ||_p$$

* **Current Implementation:** Uses the $L_2$ norm (Euclidean distance on the embedding).
* **Theoretical Ideal:** Should use the $p$-adic Metric $d_p(x, y) = p^{-v_p(x-y)}$ to reflect hierarchical violation severity.

---

### 3. The Composition Algebra (Aggregation Rules)

Currently, the code sums energy (`total_energy += step_energy`). This implies a specific assumption about error independence. We formalize the **Energy Aggregation Operator** $\bigoplus$.

#### 3.1 Sequential Composition (Time)

For a proof path $\tau = (a_1, a_2, \dots, a_T)$:

$$E_{path}(\tau) = \sum_{t=1}^T \gamma^{T-t} E_{opt}(s_t, a_t)$$

**Semantics:** Additive cost with discount factor $\gamma$. This encourages fixing errors early (since early errors propagate).

#### 3.2 Logical Composition (Space/Branches)

For a branching proof (e.g., Case Analysis $C_1 \land C_2$):

$$E_{branch}(C_1, C_2) = \max(E(C_1), E(C_2))$$

**Semantics:** Ultrametric Property. A proof is only as valid as its weakest branch.

> **Note:** The current Rust implementation uses summation (+) for branches, which is a "probabilistic approximation" (Total Error Mass). The rigorous logical definition requires `max`.

---

### 4. Energy as a Lyapunov Candidate

To guarantee VAPO convergence, $E_{opt}$ must satisfy the **Lyapunov Conditions**:

1.  $E_{opt}(s) \ge 0$ everywhere.
2.  $E_{opt}(s) = 0 \iff s$ is logically valid.
3.  $\dot{E} = E(s_{t+1}) - E(s_t) \le 0$ under the control law $u = \text{VAPO}(s)$.

**Implication:** VAPO is essentially a "Descent Algorithm" on the $E_{opt}$ surface. The "structural hole" is closed by ensuring $E_{opt}$ is convex (or at least quasi-convex) in the neighborhood of the solution, which Theorem 5.7 (Controllability) guarantees via the Bias linear action.

---

### 5. Type Definition Proposal

The Rust type for Energy should be enriched to support these semantics:

```rust
enum EnergySignal {
    /// Perfectly Valid
    Zero,
    /// Metric Violation (with magnitude)
    Scalar(f64),
    /// Critical Logical Failure (NaN/Inf)
    Infinity,
    /// Vector of independent constraints (for multi-objective)
    Vector(Vec<f64>),
}

impl EnergySignal {
    fn aggregate_sequential(&self, other: &Self) -> Self { ... } // Add
    fn aggregate_logical(&self, other: &Self) -> Self { ... }    // Max
}
```
