# THEORY PATCH: Geometry of Non-Smooth Control

Defining Manifolds, Cones, and Retractions for Evolver

## 1. The Geometric Interface Gap

Previous patches defined the topology (Bundle) and dynamics (Markov), but lacked the metric geometry.

* **Problem:** $S_{t+1} = S_t + \delta$ is undefined because logical states cannot be added.
* **Problem:** "Direction of improvement" is ambiguous at singular points (e.g., when a proof branch splits).

We formalize the system using **Riemannian Geometry on Stratified Spaces**.

## 2. The Ambient Manifold: Stratified Space

We embed the STP State Space into a continuous Stratified Manifold $\mathcal{M}$.

$$\mathcal{M} = \bigcup_{k} \Sigma_k$$

* **Strata ($\Sigma_k$):** Each stratum represents a "logical isomorphism class" (e.g., all proofs that have successfully defined $n$). Within a stratum, the metric is smooth (changing variable values).
* **Singularities:** The boundaries between strata represent discrete logical steps (Define, Branch, QED). These are "folds" or "corners" in the manifold.

## 3. The Tangent Structure: The Clarke Tangent Cone

Since $\mathcal{M}$ is non-smooth, we cannot define a linear Tangent Space $T_S \mathcal{M}$.
Instead, we define the **Tangent Cone** $C_S \mathcal{M}$ (specifically, the Clarke Tangent Cone for optimization).

$$C_S \mathcal{M} = \{ v \in \mathbb{R}^V \mid \exists t_n \downarrow 0, x_n \to S \text{ s.t. } S + t_n v \in \mathcal{M} \}$$

* **Interpretation:** The Bias Vector $\vec{b}$ is a vector in the Ambient Embedding Space.
* **Validity:** $\vec{b}$ is effective only if its projection falls within the Tangent Cone $C_S \mathcal{M}$. If it hits the wall of the cone, the logic "jams."

## 4. The Operator: Retraction (Projection)

The fundamental operation of Evolver is not addition, but **Retraction**.
We define the Retraction map $R: T \mathcal{M} \to \mathcal{M}$ that maps a tangent vector back onto the manifold.

$$S_{new} = R_S(P \cdot \vec{b})$$

In our context, the ActionDecoder + STP Transition is the Retraction map.

* **First Order:** It moves along the geodesic defined by $\vec{b}$.
* **Correction:** It projects the result back to the nearest valid stratum (Energy minimization).

## 5. Curvature and Stability (Second Order)

We define the **Logical Curvature** $\kappa(S)$ via the Hessian of the Energy function $\nabla^2 E$.

* **Flat Space ($\kappa \approx 0$):** Within a stratum (e.g., changing numerical constants). Small bias changes lead to proportional state changes.
* **High Curvature ($\kappa \gg 0$):** Near phase transitions (e.g., just before closing a proof branch). Small bias changes can cause the state to snap to a completely different topology.

**VAPO's Geometric Interpretation:**
VAPO acts as a Riemannian Trust Region method.
1. It approximates the Retraction $R_S$ locally.
2. It implicitly estimates the curvature $\kappa$ (via the success/failure of perturbations) to adjust the step size (Valuation Level).

## 6. Strict Mathematical Interfaces

The system interfaces must reflect these geometric objects.

```rust
trait ManifoldOps {
    /// Returns the set of feasible directions at the current singularity
    /// (Generators of the Tangent Cone)
    fn tangent_cone_generators(&self, state: &STPState) -> Vec<Direction>;

    /// The Retraction Map: Projects a vector in ambient space onto the manifold
    /// Returns the new state and the effective distance moved
    fn retract(&self, state: &STPState, vector: BiasVector) -> (STPState, f64);
}
```

This completes the geometric picture: Evolver is controlling a particle on a rugged, stratified landscape by applying forces in the ambient space and relying on retraction to keep it on the surface.
