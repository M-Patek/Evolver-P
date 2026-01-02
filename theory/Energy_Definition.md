# Unified Energy Metric: The Split Projection Model

> "The Will climbs the smooth hill; the Truth resides in the sharp valley."

## 1. The Paradox: Continuity vs. Chaos

In designing the Evolver's projection function $\Psi$, we encounter two contradictory requirements:

* **Requirement A: Lipschitz Continuity (For Search)**
    The VAPO optimizer requires that small perturbations in the algebraic state $S$ result in small changes in the output Energy $E$.
    $$||\Delta S|| < \delta \implies ||\Delta E|| < \epsilon$$
    Without this, the Cayley Graph is just random noise, and no heuristic search is possible.

* **Requirement B: Avalanche Effect (For Verification)**
    Logical Truth must be rigorous. A "slightly wrong" logic is still "wrong." Furthermore, for security (Proof of Will), the mapping should be non-invertible and strictly sensitive to exact state configurations.
    $$\Delta S \neq 0 \implies \Psi(S) \text{ changes significantly (Bit Flip)}$$

**The Resolution:** We split the Materialization process into two distinct projections, separating the Search Objective from the Verification Objective.

---

## 2. The Split Architecture

### 2.1 $\Psi_{topo}$: The Heuristic Projection (Will Layer)

* **Function:** $\Psi_{topo}(S) = \text{Bucket}(\text{ModularFeatures}(S))$
* **Input:** Algebraic State $S$.
* **Mechanism:** Extracts smooth geometric features ($\cos, \sin, \log y$) and maps them to coarse-grained buckets.
* **Property:** Lipschitz Continuous (Locally). Small changes in $S$ usually result in the same bucket or adjacent buckets.
* **Role:** Provides the "Gradient Sense". It defines the Basin of Attraction.

### 2.2 $\Psi_{exact}$: The Exact Projection (Truth Layer)

* **Function:** $\Psi_{exact}(S) = \text{Hash}(\text{Canonical}(S))$
* **Input:** Algebraic State $S$.
* **Mechanism:** Strictly reduces $S$ to its canonical form $(a, b, c)$ and performs a cryptographic hash on the structural integers.
* **Property:** Avalanche / Discrete. Even the slightest deviation in the algebraic structure (e.g., $a \pm 1$) results in a completely different, pseudo-random output.
* **Role:** Defines the Barrier Function. It validates the specific Proof of Work.

---

## 3. The Unified Energy Function

The total system energy $J(S)$ is a weighted sum of these two layers:

$$J(S) = \underbrace{\mathcal{V}_{STP}(\Psi_{exact}(S))}_{\text{Discrete Barrier (Avalanche)}} + \lambda \cdot \underbrace{||\Psi_{topo}(S) - \Psi_{target}||^2}_{\text{Continuous Residual (Lipschitz)}}$$

### Dynamics of Optimization

1.  **Exploration Phase (Guided by $\Psi_{topo}$):**
    When the system is logically invalid (High Barrier), the optimizer follows the gradient of the Continuous Residual. It brings the state into the correct "Geometric Neighborhood."

2.  **Lock-in Phase (Enforced by $\Psi_{exact}$):**
    Once the state is structurally close, the optimizer must find the exact algebraic configuration that satisfies the STP constraints. Any "almost correct" state (in the same bucket but wrong structure) will be rejected by the Avalanche Barrier.

This ensures that Intelligence (Low Energy) is strictly bound to Work (Finding the Exact State), preventing collision attacks based on geometric proximity.
