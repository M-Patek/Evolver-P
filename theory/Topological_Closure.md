# THEORY PATCH: The Neuro-Symbolic Fibration

Closing the Structural Hole between $\mathbb{R}^V$, $A$, and $S$

## 1. Problem Identification: The Type Gap

The current system operates on three disjoint sets without a unifying morphism:

* **The Manifold** $L \cong \mathbb{R}^V$: The space of raw logits (Log-probabilities).
* **The Lattice** $A$: The discrete set of valid ProofActions.
* **The Module** $S$: The STP algebraic state space.

**The Hole:** The mappings $f: L \to A$ (Decoding) and $g: A \to S$ (Transition) are currently treated as external engineering functions. We lack a closed topological definition where $L, A, S$ are objects of the same Category $\mathcal{C}_{Evolver}$.

---

## 2. The Mathematical Closure: Fiber Bundles

To close this gap, we redefine the system as a Principal Bundle structure.

### 2.1 The Base Space: Algebraic Truth ($S$)
Let the STP State Space $S$ be the Base Manifold (or Base Scheme).
Points $s \in S$ are valid algebraic states (e.g., "n is Odd").
Movement in $S$ is governed by the matrix equation $s_{t+1} = M \ltimes u \ltimes s_t$.

### 2.2 The Discrete Fiber: Action Schemas ($A|_s$)
At any state $s \in S$, not all actions in the universe $A_{univ}$ are topologically valid.
We define the Action Fiber at $s$ as:

$$A|_s = \{ a \in A_{univ} \mid \text{Energy}(s, a) < \epsilon \}$$

This creates a **Sheaf of Actions** over $S$.

### 2.3 The Continuous Bundle: Logit Distributions ($L|_s$)
We redefine Logits not as a detached vector space, but as a Probability Measure over the Fiber $A|_s$.
The Logit Space $L$ is the total space of a bundle $E \xrightarrow{\pi} S$, where the fiber at $s$ is the tangent space of the probability simplex over valid actions:

$$L|_s \cong T(\Delta^{|A|_s| - 1})$$

---

## 3. The Unifying Morphism: The "Section"

In this formalism, the "Generator" (LLM) is no longer a black box. It is a **Section** (截面) of the bundle.

$$\sigma: S \to L$$

The generator observes the current algebraic state $s$ and selects a distribution $l \in L|_s$.

### 3.1 The Projection (Decoder): Voronoi Retraction
To strictly define the morphism $L \to A$ without breaking the bundle topology, the Decoder $\Pi$ must be defined as a **Deterministic Retraction** associated with a Voronoi Tessellation.

We verify that $L|_s \cong \mathbb{R}^V$. For each valid action $a_i \in A|_s$, we associate a characteristic vector $v_{a_i}$ (e.g., one-hot encoding). The decoder partitions the continuous fiber $L|_s$ into **Action Cells**:

$$Cell(a_i) = \{ z \in L|_s \mid \forall j \neq i, \langle z, v_{a_i} \rangle > \langle z, v_{a_j} \rangle \}$$

The Decoder $\Pi$ is the map that collapses the entire cell to its singularity:

$$\Pi(z) = a_i \iff z \in Cell(a_i)$$

* **Mathematical Object:** $\Pi$ is a discontinuous section projection (Deterministic Retraction).
* **Measure Theory:** It pushes forward the Lebesgue measure on $L|_s$ to a Dirac mass distribution on $A|_s$.
* **Thermodynamics:** This corresponds to the limit of the Gibbs measure (Softmax) as Temperature $T \to 0$.

### 3.2 The Transition (STP Dynamics)
The STP update is a functor $F: A \to \text{End}(S)$.

$$s_{t+1} = F(\Pi(\sigma(s_t))) \cdot s_t$$

---

## 4. Closing the Loop: The Bias Connection

Here is where VAPO (Bias Controller) fits into the topology.

The structural hole exists because $\Pi$ (Argmax/Voronoi) is non-invertible and discontinuous at cell boundaries. The **Bias Vector** $\vec{b}$ acts as a **Connection Form** (联络形式) on the bundle.

It defines a Parallel Transport operation via translation. If the generator's section $\sigma(s)$ lands in a "high energy" (invalid) Voronoi cell, the Bias $\vec{b}$ translates the point across the boundary into a valid cell:

$$\sigma'(s) = \sigma(s) +_{\text{fiber}} \vec{b}$$

**Closure Theorem:**
The system is structurally closed if and only if for every state $s$ and every valid target $s_{next}$, there exists a connection $\vec{b}$ such that:

$$F(\Pi(\sigma(s) + \vec{b})) \cdot s = s_{next}$$

This is exactly what Theorem 5.7 (Controllability) proves. The Bias Channel closes the topological hole by ensuring that the fiber $L|_s$ covers the entire neighborhood of valid transitions in $S$ via translation.

---

## 5. Architectural Implication

We must strictly enforce that logits are NOT just `Vec<f64>`, but typed objects aware of their base state $S$.

**Proposed Rust Type System:**

```rust
struct BundleState<S> {
    base: S,                  // The STP State
    fiber: ActionSchema<S>,   // Valid actions at this state
}

struct LogitSection<'a, S> {
    bundle: &'a BundleState<S>,
    distribution: Vec<f64>,   // R^V constrained to the fiber
}

trait Morphism<S> {
    // The map L -> A must be aware of S to be valid
    fn project(section: LogitSection<S>) -> ProofAction;
}
```

This ensures that we never process logits "in a vacuum" ($\mathbb{R}^V$), but always as tangent vectors attached to a logical state.
