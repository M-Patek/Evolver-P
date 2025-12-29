# THEORY PATCH: Constraints on the Projection Operator $P$
## Defining the Geometry of Low-Rank Control

### 1. The Dimensionality Gap
The controller operates in $\mathbb{R}^k$ (Bias Space, e.g., $k=16$) while the generator operates in $\mathbb{R}^V$ (Logit Space, e.g., $V=32000$).
The projection $P: \mathbb{R}^k \to \mathbb{R}^V$ defines the **Control Subspace** $\mathcal{K} = \text{Range}(P)$.

**The Problem**: If the "Correct Logic" requires a perturbation $\Delta z \perp \mathcal{K}$, the system is uncontrollable.

---

### 2. Formal Definition of $P$
We define $P$ not as a generic linear map, but as a **Structured Isometric Embedding** satisfying specific invariants.

#### 2.1 Constraint 1: Gauge Invariance (Zero-Mean)
The Softmax function $\sigma(z)$ is invariant under translation by the vector $\mathbf{1} = [1, 1, \dots, 1]^T$:
$$\sigma(z + c\mathbf{1}) = \sigma(z)$$
Any bias energy spent in the direction of $\mathbf{1}$ is wasted ("Heat").

**Constraint**: The columns of $W$ must be orthogonal to $\mathbf{1}$.
$$\mathbf{1}^T W = 0 \implies \sum_{i=1}^V W_{i,j} = 0, \quad \forall j \in \{1 \dots k\}$$

#### 2.2 Constraint 2: Restricted Isometry Property (RIP)
To ensure that distances in Bias Space reflect distances in Logic Space (preserving VAPO's gradients), $P$ must be a near-isometry on sparse vectors:
$$(1-\delta) ||b||_2^2 \le ||Pb||_2^2 \le (1+\delta) ||b||_2^2$$

This justifies the use of a "Random Projector" via the **Johnson-Lindenstrauss Lemma**.



---

### 3. Reachability Analysis
Since $k \ll V$, we cannot reach any point in $\mathbb{R}^V$. We rely on the **Manifold Hypothesis**.

* **Assumption**: The set of valid logical states $\mathcal{S}_{valid}$ lies on a low-dimensional submanifold of $\mathbb{R}^V$ with intrinsic dimension $d \ll k$.
* **Projection Theorem**: If $W$ is a random orthogonal projection and $k > O(d \log V)$, then with high probability, the projection of the manifold onto the control space is a diffeomorphism (**Whitney Embedding Theorem** equivalent).

---

### 4. Revised Controllability Theorem
We downgrade Theorem 5.7 from "Exact Controllability" to "**$\epsilon$-Approximate Controllability**".

**Theorem 5.7 (Revised)**:
For any target distribution $p^*$ on the logical manifold $\mathcal{M}$, there exists a bias $b \in \mathbb{R}^k$ such that:
$$D_{KL}(\sigma(z + Pb) || p^*) < \epsilon$$
Provided that the "Angle" between the Control Subspace $\mathcal{K}$ and the Tangent Space $T_{p^*} \mathcal{M}$ is non-zero.



---

### 5. Implementation Requirement
The `BiasProjector` initialization in `bias_channel.rs` must be updated to enforce **Column Centering** (Constraint 2.1) and **Unit Norm Scaling** (Constraint 2.2).

```rust
// Proposed fix for BiasProjector::new()
fn orthog_projector() {
    // 1. Generate Gaussian Matrix
    // 2. Subtract Mean from each column (Enforce 1^T W = 0)
    // 3. QR Decomposition to orthogonalize columns (Enforce Isometry)
}
```
