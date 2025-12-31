# Evolver: Algebraic Logic Generation Engine
"Logic is not corrected; it is evolved."

Evolver is a native algebraic logic generation engine based on the Hyper-Tensor Protocol (HTP) and Semi-Tensor Product (STP).

Unlike traditional "Neuro-Symbolic" systems, Evolver no longer acts as a "correction sidecar" or a "probabilistic patcher" for LLMs. It is an independent generative core that "grows" mathematically certain logical paths directly by performing evolution and search on rigorous algebraic manifolds (ideal class groups).

## 🏛️ Core Architecture: The Tripartite Soul
The system architecture mimics the structure of a living organism, consisting of three core components: Soul (Algebraic Kernel), Will (Optimization Intent), and Body (Topological Form).

### 1. The Soul (Algebraic Core)
* **Code:** `src/soul/algebra.rs`
* **Mathematical Entity:** Elements in the Ideal Class Group 
$$Cl(\Delta)$$
of imaginary quadratic fields.
* **Responsibility:** Carries the "subconscious" of logic. It does not store specific tokens but rather the algebraic states 
$$(a, b, c)$$
* **Mechanism:** Driven by a seed generated from the Context Hash, the Soul undergoes deterministic chaotic evolution along group orbits.

### 2. The Will (VAPO Optimizer)
* **Code:** `src/will/optimizer.rs`
* **Algorithm:** VAPO (Valuation-Adaptive Perturbation Optimization).
* **Responsibility:** Searching for Truth.
* **Workflow:**
    * Observes the current algebraic state.
    * Projects it into logical actions and calculates the STP energy 
($$E_{STP}$$).
    * If 
$$E > 0$$
 (logical contradiction), it applies a perturbation in the algebraic space.
    * This is a discrete, non-gradient Metropolis-Hastings search process acting directly on the Soul.

### 3. The Body (Topological Projector)
* **Code:** `src/body/`
* **Mathematical Entity:** v-PuNN (Valuation-Adaptive Perturbation Neural Network) topology.
* **Responsibility:** Materialization.
* **Mechanism:** Collapses the optimized abstract algebraic state 
$$(a, b, c)$$
into human-readable digit sequences or logical action paths through Artin-like Projection and fractal unfolding.

---

## 🔄 Workflow: The Generation Loop
Evolver's execution no longer relies on external LLM logits. The process is entirely endogenous:

1.  **Inception:** User inputs the Context (string).
2.  **Seeding:** Calculates Hash(Context) as the initial kinetic energy acting on the Identity Soul.
$$S_0 = S_{id} \circ \text{Evolve}(\text{Seed})$$
3.  **Optimization:** The Will takes control. It continuously applies algebraic perturbations to 
$$S_t$$
 until it finds a state 
$$S^*$$
 where the materialized logical path satisfies the zero-energy constraint.
$$\text{Minimize } E_{STP}(\text{Materialize}(S)) \implies S^*$$
4.  **Materialization:** Unfolds the perfect algebraic state 
$$S^*$$
 into a token sequence.
$$\text{Path} = \text{Project}(S^*)$$

---

## 🛠️ Tech Stack & Features
* **Language:** Rust (Focusing on zero-cost abstractions and type safety)
* **Algebra:** `num-bigint` (Handling class group operations for large integers)
* **Optimization:** VAPO (Proprietary discrete optimization algorithm)
* **Verification:** STP Engine (Logic-physics engine based on Semi-Tensor Product)
* **Interface:** PyO3 (Providing Python bindings for easy integration)

## 📦 Quick Start (Python API)
Evolver is designed as a high-performance Python extension module.

### Compilation
```bash
# Requires Rust toolchain installed
maturin develop --release
```

### Usage Example
```python
from new_evolver import PyEvolver

# 1. Initialize the engine
# p=409 (Prime base), k=19 (Decision tree depth)
engine = PyEvolver(409, 19)

# 2. Input context (e.g., a mathematical proposition)
context = "Prove that the sum of two Odd numbers is Even."

# 3. Align
# This step involves the full process of evolution, search, and materialization.
# Returns: An STP-verified logical path (Token IDs) with zero energy.
path = engine.align(context)

print(f"Generated Logic Path: {path}")
# Output: [3, 7, 6, ...] (Corresponding to: Define n: Odd, Define m: Odd, Apply Add...)
```

---

## 🧠 Theoretical Background
### Why move from Algebra to Logic?
Traditional AI attempts to simulate logical reasoning through trained neural networks (probabilistic approximation). Evolver takes the opposite approach: we construct an algebraic space (Class Groups) isomorphic to logical structures. In this space, Truth is the stable state with the lowest energy.

We don't train models to "guess" answers; we use optimization algorithms to let the answers "emerge" from the algebraic structure.

* **Semi-Tensor Product (STP):** Provides the physical laws of conservation for logic.
* **Class Groups:** Provide a sufficiently complex and continuous search space (manifold) allowing discrete logic to be optimized.

---

## 📜 License
M-Patek PROPRIETARY LICENSE Copyright © 2025 M-Patek.
See the LICENSE file for details.
