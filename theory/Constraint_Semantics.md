# THEORY PATCH: Semantics of the Constraint Set

Disambiguating $\mathcal{C}$ in the Neuro-Symbolic Bundle

## 1. The Semantic Ambiguity

The term "Constraint" currently conflates three distinct mathematical structures:

* **Feasibility:** Is action $a$ valid at state $s$? ($C \subseteq S \times A$)
* **Admissibility:** Is the sequence $a_1 \dots a_n$ valid? ($C \subseteq A^*$)
* **Invariance:** Does the transition preserve truth? ($T(s, a) \in S_{true}$)

To resolve this, we define the Constraint Set $\mathcal{C}$ as an Algebraic Variety on the Bundle.

## 2. Formal Definition: The Local Variety

Let $\mathcal{B} = S \times A$ be the total space of the State-Action bundle.
We define the Local Constraint Variety $\mathcal{V}_{loc}$ as the kernel of the Energy functional:

$$\mathcal{V}_{loc} = \{ (s, a) \in S \times A \mid E(s, a) = 0 \}$$

This defines a Fibered Submanifold of valid transitions.

**Interpretation:** The set $\mathcal{C}$ is NOT a static subset of $A$. It is the projection of $\mathcal{V}_{loc}$ onto the fiber $A|_s$.

**Code Mapping:**
* `ProofAction::Define` checks if $a$ fits the static topology of $S$.
* `ProofAction::Assert` checks if $(s, a)$ lies on the truth surface.

## 3. The Global Constraint: Trace Language

The dynamics induce a Language $\mathcal{L} \subseteq A^*$ defined by the valid traces on $\mathcal{V}_{loc}$.

A sequence $\tau = (a_1, \dots, a_T)$ is in $\mathcal{C}_{global}$ iff there exists a state sequence $s_0, \dots, s_T$ such that:

$$\forall t, (s_t, a_{t+1}) \in \mathcal{V}_{loc} \quad \text{and} \quad s_{t+1} = \Phi(s_t, a_{t+1})$$

This formalizes the constraint as a Subshift of Finite Type (or Context-Sensitive Language, depending on STP complexity).

## 4. STP Specifics: Commutative Diagram Constraints

In STP, "Applying a Theorem" is a specific type of constraint: Commutative Invariance.

Let $D_s$ be the derivation diagram at state $s$.
Let $f_{thm}: S \to S$ be the transformation implied by the theorem.
The constraint is that the computational path must commute with the logical path:

$$Constraint_{apply}(s, a) \iff || s_{next}^{computed} - s_{next}^{logic} || = 0$$

Where $s_{next}^{logic} = M_{thm} \ltimes s$.

## 5. Type System Implication

To strictly enforce this, the Rust `STPContext` should expose the constraint check as a Membership Test on the Variety.

```rust
trait ConstraintManifold {
    /// Checks if the point (s, a) lies on the Zero-Energy Variety
    fn contains(&self, state: &STPState, action: &ProofAction) -> bool;

    /// Returns the Tangent Space of valid modifications at (s, a)
    /// Used by VAPO to project gradients.
    fn tangent_space(&self, state: &STPState, action: &ProofAction) -> Vec<BiasVector>;
}
```

This clarifies that VAPO is performing Manifold Optimization: moving the point $(s, \hat{a})$ onto the surface $\mathcal{V}_{loc}$.
