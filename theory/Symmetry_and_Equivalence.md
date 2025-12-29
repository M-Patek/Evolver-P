# THEORY PATCH: Symmetry and Equivalence Classes

Resolving the Representation vs. Semantics Conflict

### 1. The Representation Trap

The Action Space $A$ contains redundancy.
* **Commutativity:** `Assert(a = b)` vs `Assert(b = a)`.
* **Path Independence:** Proving Lemma X then Y vs Lemma Y then X.

If $E(a) \neq E(a')$ for $a \sim a'$, the Energy Landscape becomes artificially rugged. The controller might waste iterations optimizing "syntax" rather than "logic."

### 2. Formal Definition: Semantic Equivalence

We define an equivalence relation $\sim$ on the Action Fiber $A|_s$.
Two actions $a, a'$ are **Semantically Equivalent** ($a \sim_s a'$) iff they induce the same state transition (up to isomorphism):

$$M_{STP}(a) \ltimes s \cong M_{STP}(a') \ltimes s$$

This induces a **Quotient Space** $\mathcal{A} = A / \sim$.

### 3. The Gauge Invariance Principle

The Energy Functional $E$ must be a **Class Function** on the quotient space.
**Constraint:**

$$\forall a, a' \in A: a \sim a' \implies E(s, a) = E(s, a')$$

This is the Gauge Invariance of the logical field. The specific syntactic choice (the "gauge") should not affect the physical potential (the energy).

### 4. Implementation: Canonicalization Strategy

To enforce invariance computationally, we introduce a **Canonicalization Operator** $\mathcal{K}: A \to A$.

$$\mathcal{K}(a) = \text{argmin}_{x \in [a]} \text{Lex}(x)$$

* **Logic:** Before calculating Energy, map the action to its canonical form (e.g., sort arguments of commutative operators, normalize variable names).
* **Dynamics:**

$$E_{impl}(s, a) := E_{core}(s, \mathcal{K}(a))$$

### 5. Geometry of the Solution Set

This implies that the "Target" for VAPO is not a single point in Logit Space $\mathbb{R}^V$, but a **Union of Polyhedra**.

Let $S_{valid} \subset \mathcal{A}$ be the set of valid semantic moves.
The solution set in Logit Space is the union of the Voronoi cells of all representatives:

$$\mathcal{Z}_{solution} = \bigcup_{[a] \in S_{valid}} \bigcup_{x \in [a]} Cell(x)$$

**Optimality Consequence:**
VAPO works better than expected because the "Bullseye" is larger. The controller only needs to hit any syntactic variation of the truth.

### 6. Code Update Recommendation

The `STPContext` needs a normalization layer.

```rust
impl ProofAction {
    fn canonicalize(&self) -> Self {
        match self {
            // Example: Sort inputs for commutative theorems
            ProofAction::Apply { theorem_id, inputs, .. } if is_commutative(theorem_id) => {
                let mut sorted_inputs = inputs.clone();
                sorted_inputs.sort();
                ProofAction::Apply { inputs: sorted_inputs, ..*self }
            },
            _ => self.clone()
        }
    }
}
```
