# HTP Implementation Strategy: Decoupling Logic from Cryptography

**Core Philosophy:** "The Control Law is invariant under the choice of the underlying Group."

This document outlines the roadmap from the current logical prototype (Phase 1) to the cryptographic final form (Phase 2), with a special focus on the role of the Neural Guide.

## 1. The Abstraction of Chaos

The Evolver architecture separates concerns:

* **The Controller (VAPO):** Observes energy and outputs bias.
* **The Plant (HTP Core):** Accepts actions and evolves state.

### Why the Specific Group Doesn't Matter (Yet)

The Bias Controller relies on Controllability, not Differentiability.
The controller treats the HTP Core as a "Black Box" function $F(x)$:

$$
State_{next} = F(State_{current}, Action + Bias)
$$

Whether $F(x)$ is a simple Matrix Multiplication (Phase 1) or a Class Group Operation (Phase 2), the control problem remains a Discrete Search Problem.

## 2. Phase 1.5: Neural Guidance (The "Proposer" Update)

### Crucial Correction:
In previous iterations, it was tempting to treat the Controller as a gradient-descent optimizer. However, since the STP Energy surface is Discrete (Step Functions) and Non-Differentiable, gradients do not exist.

### The Solution: Neural-Guided Search
Instead of backpropagation, we implement a Predictor-Corrector architecture:

**The Transformer (Predictor):**

* Acts as a Heuristic Function or Intuition.
* Learns the mapping:

$$
\text{Context} \to P(\vec{b}_{\text{optimal}})
$$

* **Output:** A "Proposal" (A starting point for the search).

**VAPO (Corrector):**

* Acts as the Solver.
* Takes the Proposal and performs local stochastic search (Metropolis-Hastings) to find the exact zero-energy solution.

### Why this works:

* **Search Space Reduction:** The Transformer reduces the search volume from "Infinite" to "Local Neighborhood".
* **Hardness Preservation:** The actual verification logic remains in the rigorous STP engine, ensuring the Transformer's "guesses" are never trusted blindly.

## 3. Phase 2: Cryptographic Hardness (Target)

* **Goal:** Ensure the history cannot be rewritten (Time Security).
* **Time Operator:** Composition of quadratic forms

$$
(a, b, c)
$$

* **Hidden Order:** Rely on the difficulty of finding class number

$$
h(\Delta)
$$

Because the Intuition Engine (Transformer) only provides proposals and does not touch the internal algebra, switching from Matrix to Class Groups does not break the neural network. The network simply learns a new proposal distribution for the new algebraic structure.

## 4. Conclusion

We are building a Neurally-Guided Symbolic Search System.

* **Logic:** Hard, Discrete, Algebraic.
* **Control:** Soft, Probabilistic, Heuristic.

The Transformer is the map; VAPO is the compass; STP is the terrain.
