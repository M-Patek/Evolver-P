# THEORY PATCH: Unification of Time Scales and Trajectories

From Token-Level Impulse to Sequence-Level Control

### 1. The Granularity Conflict

The controller $\Phi$ is implemented as align(logits), implying a per-token operation $z \in \mathbb{R}^V$.
However, a logical ProofAction (e.g., Define { ... }) is a complex structure spanning multiple tokens $z_{1:K}$.

If $\Phi$ is purely local, it cannot fix structural errors (JSON syntax, multi-token coherence).

If $\Phi$ is purely global, the search space $\mathbb{R}^{T \times V}$ is too large for VAPO.

### 2. Formal Definition: Hierarchical Time

We define two distinct time scales:

* **Micro-Time ($\tau$):** The index of the auto-regressive generation (Token step).
* **Macro-Time ($t$):** The index of the STP State transition (Logical step).

The trajectory is a sequence of Logical Frames:

$$\mathcal{T} = (\mathbf{Z}_1, \mathbf{Z}_2, \dots, \mathbf{Z}_N)$$

Where each frame $\mathbf{Z}_t \in \mathbb{R}^{K_t \times V}$ corresponds to the tokens constituting the $t$-th Action.

### 3. The Control Object: The Logical Frame

The Controller $\Phi$ does not map $\mathbb{R}^V \to \mathbb{R}^V$.
It maps a Predicted Frame to a Controlled Frame:

$$\Phi: \mathbb{R}^{K \times V} \times \mathcal{S} \to \mathbb{R}^{K \times V}$$

* **Input:** The Generator's "intent" for the next full logical step (e.g., predicted via lookahead or iterative refinement).
* **Output:** A biased sequence of logits that decodes to a valid ProofAction.

### 4. Control Strategy: Receding Horizon Control (MPC)

Since generating the full frame $Z_t$ before control is computationally expensive (requires full rollout), $\Phi$ is formally a Model Predictive Controller.

At each micro-step $\tau$ within macro-step $t$:

1.  **Predict:** Estimate the remainder of the current logical frame (Lookahead).
2.  **Optimize:** Solve VAPO for the Frame Bias $\mathbf{B}_t$ (a bias parameter shared or distinct across the frame).
3.  **Act:** Apply only the first step $\vec{b}_\tau$ of the optimal plan.
4.  **Recede:** Move to $\tau+1$, observe new state, repeat.

### 5. Mathematical Definition of $\Phi$

$\Phi$ is a functional on the Trajectory Space $\Omega = (\mathbb{R}^V)^\mathbb{N}$.

$$\Phi(\mathbf{z}_{1:\infty}, S) = \mathbf{z}'_{1:\infty}$$

Subject to the **Causality Constraint**:

$$\mathbf{z}'_\tau \text{ depends only on } \mathbf{z}_{1:\tau} \text{ and } S$$

And the **Block-wise Validity Constraint**:

$$\forall t, \text{Dec}(\mathbf{z}'_{\text{frame}(t)}) \in \mathcal{C}|_{S_t}$$

### 6. Implementation Implication

The BiasController must theoretically maintain a Short-Term Memory or Lookahead Buffer.

* **Current Simplification:** If the code assumes 1 Token = 1 Action (e.g., via special embeddings), then Micro = Macro.
* **Full Implementation:** VAPO must act on the Hidden State of the LSTM/Transformer to bias the entire rollout of the next action, not just the immediate token.

```rust
struct LogicalFrameControl {
    // The bias is not just a vector, but a "Plan" for the sequence
    bias_plan: Vec<BiasVector>, 
    horizon: usize,
}
```
