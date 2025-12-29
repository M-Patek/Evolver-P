# THEORY PATCH: Stochasticity and Uncertainty

Formalizing the System as a Controlled Markov Process

### 1. The Sampling Gap

The relationship between Logits $z$ and Action $a$ is ambiguous:

* **Case A (Deterministic):** $a = \text{argmax}(z)$. VAPO optimizes a function.
* **Case B (Stochastic):** $a \sim \text{Softmax}(z/T)$. VAPO optimizes a random variable.

Without choosing one, "optimizing energy" is ill-defined. Minimizing $E(\text{sample})$ is noisy; minimizing $E(\text{argmax})$ ignores variance.

### 2. Formal Definition: Controlled Markov Kernel

We define the system dynamics as a Controlled Stochastic Transition Kernel $\mathcal{K}$.

$$P(S_{t+1} | S_t, \vec{b}_t) = \sum_{a \in \mathcal{A}} \mathbb{I}[S_{t+1} = \Phi_{STP}(S_t, a)] \cdot \text{Softmax}(z(S_t) + P\vec{b}_t)_a$$

This explicitly places the system in the realm of Stochastic Control.

### 3. The Certainty Equivalence Principle (VAPO's Strategy)

Computing the full expectation $\mathbb{E}_{a}[E(S, a)]$ is computationally prohibitive (requires summing over all actions).

VAPO adopts the **Certainty Equivalence Approximation**: It assumes that the random variable $a$ can be replaced by its Mode (deterministic estimator) for the purpose of control optimization.

**Optimization Objective (Mode-Seeking):**

$$\vec{b}^* = \text{argmin}_{\vec{b}} \mathcal{E}_{STP}(S_t, \text{argmax}(\text{Softmax}(z + P\vec{b})))$$

**Justification:** We are performing MAP (Maximum A Posteriori) estimation on the joint distribution of logic and language. By forcing the peak of the distribution into the valid region, we maximize the probability of sampling a valid action.

### 4. Definition of "Correctness": $(\epsilon, \delta)$-Validity

Since the system allows sampling ($T > 0$), we cannot guarantee $E=0$ absolutely (unless $T \to 0$).
We define the target correctness property probabilistically:

A state $S$ is $(\epsilon, \delta)$-Valid if there exists a bias $\vec{b}$ such that:

$$P_{a \sim Q_{\vec{b}}} (\text{Energy}(S, a) > \epsilon) < \delta$$

* $\epsilon$: Tolerance for metric deviation.
* $\delta$: Failure probability (Risk).

VAPO's "Success" condition (returning $E=0$ on argmax) serves as a proxy for minimizing $\delta$.

### 5. Implementation implication

The `ActionDecoder` trait must distinguish between `decode_deterministic` (for VAPO loop) and `decode_stochastic` (for final execution).

```rust
trait ActionDecoder {
    /// Returns the Mode (Argmax). Used by VAPO inner loop.
    fn decode_deterministic(&self, logits: &[f64]) -> ProofAction;

    /// Returns a Sample. Used for "Creativity" or diverse exploration.
    fn decode_stochastic(&self, logits: &[f64], temperature: f64) -> ProofAction;
}
```

This ensures VAPO converges stably (optimizing a fixed function) while the final system retains the ability to be creative.
