# THEORY PATCH: Dynamics of Context Evolution

## Formalizing the Transition Relation $\mathcal{R}$

### 1. The Dynamical Ambiguity

The expression $S_{t+1} = F(S_t, a_t)$ assumes a linear, deterministic trajectory. However, logical proofs involve:

* **Branching:** $S_t$ splits into independent sub-contexts $S_{t+1}^{(L)}$ and $S_{t+1}^{(R)}$.
* **Scope Management:** Local variables in a branch must not leak.
* **Indeterminacy:** The generator may attempt multiple strategies (Backtracking).

Therefore, the dynamics cannot be a simple function on valuations.

### 2. State Space Redefinition: The Configuration

We distinguish between the Valuation (Algebraic values) and the Configuration (Control state).

* **Micro-State (Valuation)** $\sigma \in \Sigma$: The map of variables to STP tensors (e.g., $\{ n \mapsto Odd \}$).
* **Macro-State (Configuration)** $\mathcal{S} \in \mathbb{S}$: A tuple $(\sigma, \Gamma, \tau)$, where:
    * $\sigma$: Current valuation.
    * $\Gamma$: The Context Stack (preserving parent scopes).
    * $\tau$: The Proof Tree Zipper (current focus in the proof topology).

### 3. The Transition Relation $\mathcal{R}$

The evolution is defined as a relation $\mathcal{R} \subseteq \mathbb{S} \times A \times \mathbb{S}$.

#### 3.1 Linear Transitions (Define/Apply/Assert)

For deterministic steps, the relation is functional:

$$(\mathcal{S}, a, \mathcal{S}') \in \mathcal{R} \iff \sigma' = M_{STP}(a) \ltimes \sigma \land \Gamma' = \Gamma$$

#### 3.2 Branching Transitions (Topology Change)

The Branch action induces a 1-to-N transition.
Let $a = \text{Branch}(C_1, C_2)$. This splits the timeline:

$$\mathcal{S} \xrightarrow{a} \{ \mathcal{S}_{C1}, \mathcal{S}_{C2} \}$$

* **Push Dynamics:** The system enters a "Superposition State" where the transition relation branches.
* **Stack Logic:** The parent valuation $\sigma$ is pushed to $\Gamma$. $\sigma_{C1}$ and $\sigma_{C2}$ evolve independently.

#### 3.3 Merging Dynamics (QED/Pop)

When a branch closes ($a = \text{QED}$), the relation performs a Logical Join:

$$(\{ \mathcal{S}'_{C1}, \mathcal{S}'_{C2} \}, \text{QED}, \mathcal{S}_{merged}) \in \mathcal{R}$$

Where $\sigma_{merged}$ is valid if and only if:

$$\text{Energy}(\mathcal{S}'_{C1}) = 0 \land \text{Energy}(\mathcal{S}'_{C2}) = 0$$

### 4. Trajectory as a Tree

The "Trajectory" is not a sequence $s_0, s_1, \dots$, but a Labeled Tree $T$.
The Generator's output (Sequence of Actions) is a Linearization (e.g., Pre-order Traversal) of $T$.

Let $\pi: A^* \to T$ be the parsing map.
The system is valid iff the generated linear sequence $u \in A^*$ maps to a tree $T$ such that every edge in $T$ satisfies the local STP relation $\mathcal{R}$.

### 5. Implementation Strategy: The Stack Machine

To align the code with this formalism, STPContext must explicitly model the Stack Machine:

```rust
struct ContextFrame {
    valuation: HashMap<String, Tensor>,
    goals: Vec<Tensor>, // Expected state for this branch
}

struct STPContext {
    current: ContextFrame,
    stack: Vec<ContextFrame>, // \Gamma
}

impl STPContext {
    fn transition(&mut self, action: ProofAction) {
        match action {
            Branch { .. } => self.push_stack(),
            QED => self.pop_and_merge(), // \oplus_logic
            _ => self.update_current(action),
        }
    }
}
```

This transforms the "Non-deterministic Relation" into a Deterministic Push-Down Automaton (DPDA), which is computationally tractable.
