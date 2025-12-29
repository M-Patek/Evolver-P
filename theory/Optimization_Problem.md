# THEORY PATCH: The Optimization Problem Statement
## Formalizing VAPO as a Constrained Minimization

### 1. The Missing Formulation
The code implements a search loop, but fails to state the mathematical problem. 
We define the Bias Controller's task as solving the following optimization problem at each time step $t$.

---

### 2. Problem Variables
* **Decision Variable**: $\vec{b} \in \mathbb{Z}_L^k$ (Discrete Bias Vector on Torus).
* **Constants**:
    * $z_0 \in \mathbb{R}^V$: Base logits from Generator.
    * $P \in \mathbb{R}^{V \times k}$: Projection matrix.
    * $s_t$: Current STP algebraic state.

---

### 3. The Objective Function
The objective $J(\vec{b})$ is a **Composite Lagrangian**:
$$J(\vec{b}) = \underbrace{\mathcal{E}_{STP}(s_t, \Pi(z_0 + P\vec{b}))}_{\text{Logical Violation}} + \lambda \cdot \underbrace{\mathcal{V}_{p}(\vec{b})}_{\text{Hierarchical Cost}}$$

* **$\Pi$**: The Voronoi Decoder (Argmax).
* **$\mathcal{E}_{STP}$**: The Energy functional ($0$ if valid, $>0$ if invalid).
* **$\mathcal{V}_{p}$**: The **p-adic Valuation Cost**.
    * We prefer "fine-tuning" (high valuation, small $p$-adic norm) over "structural changes".
    * $\mathcal{V}_{p}(\vec{b}) = \sum -v_p(b_i)$ (Approximation).



---

### 4. Constraints
1.  **Hard Constraint (Logic)**: The search must eventually satisfy $\mathcal{E}_{STP} = 0$.
2.  **Domain Constraint**: $\vec{b} \in [0, L)^k$ (Torus boundary).
3.  **Budget Constraint**: $N_{iter} \le N_{max}$ (Real-time limit).

---

### 5. VAPO as a Solver Algorithm
Since $\Pi$ is discontinuous (step function), $J(\vec{b})$ is **non-differentiable and terraced**. Gradient Descent is inapplicable.
VAPO (Valuation-Adaptive Perturbation Optimization) is formally defined as a **Variable Neighborhood Search (VNS)** algorithm:

* **State**: Current solution $\vec{b}_{curr}$.
* **Neighborhood Structure $\mathcal{N}_k(\vec{b})$**: Defined by p-adic valuation levels.
    * $\mathcal{N}_0$: Perturb lowest bits (Global jumps).
    * $\mathcal{N}_{max}$: Perturb highest bits (Local jitter).
* **Adaptive Rule**:
    * If $J(\vec{b})$ is high, select $\mathcal{N}_0$ (Coarse search).
    * If $J(\vec{b})$ is low but $>0$, select $\mathcal{N}_{high}$ (Fine search).



---

### 6. Code Implication: Explicit Cost Function
The Rust implementation should explicitly separate the "Cost Evaluator" from the "Search Strategy".

```rust
struct OptimizationProblem<'a> {
    base_logits: &'a [f64],
    stp_ctx: &'a STPContext,
}

impl<'a> OptimizationProblem<'a> {
    /// Evaluates J(b)
    fn evaluate(&self, bias: &BiasVector) -> f64 {
        let action = self.decode(bias);
        let energy = self.stp_ctx.energy(&action);
        let regularization = bias.p_adic_norm();
        energy + LAMBDA * regularization
    }
}
```
