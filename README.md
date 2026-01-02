# Evolver: A Formally Verifiable Evolutionary Solver
> "Empower mathematical models with an evolutionary will."

Evolver is a universal system evolution framework based on **Semi-Tensor Product (STP)**. It models complex system dynamics as rigorous algebraic structures and utilizes valuation-adaptive perturbation algorithms to find optimal evolution paths within topological spaces.

Unlike traditional black-box optimizers, Evolver emphasizes **Formal Verifiability**.

---

## Core Philosophy

### 1. The Trinity Architecture
Evolver decouples systems into three orthogonal dimensions:

* **Body (Structure):** Defines the topological space and state representation of the system. It is the carrier of evolution.
* **Soul (Laws):** Defines the dynamical rules and constraints of the system. Based on STP algebra, it ensures the **Logical Soundness** of system evolution.
* **Will (Intent):** Defines the direction of evolution. Through **Valuation-adaptive Perturbation (v-PuNNs)**, it conducts purposeful exploration within the solution space.

### 2. Trust Model
We do not promise "Correct-by-Construction"; instead, we provide a layered trust model:

* **Soundness:** Every step of the system's evolution strictly follows predefined algebraic laws (no illegal states can occur).
* **Verifiability:** The system’s evolutionary path generates a cryptographically secure **Trace**, allowing third parties to verify the authenticity of the results at a low cost.

---

## Installation & Usage

### Prerequisites
Evolver is built with **Rust**. Please ensure your local environment has the Rust toolchain (1.70+) installed.

### Building the Project
As this is private proprietary software, please ensure you have the necessary access permissions for the source code.

```bash
# Enter project root
cd evolver

# Build release version
cargo build --release
```

### Example: Defining a Simple Boolean Network
```rust
// Define state space (Body)
let topology = Topology::new(2); // 2-node network

// Define dynamical rules (Soul)
// Use STP Bridge to compile logical rules into algebraic matrices
let rules = StpBridge::compile("x1(t+1) = x1(t) AND x2(t)");

// Inject Will
// Goal: Find a path that converges to a fixed point
let optimizer = Optimizer::new()
    .with_strategy(Strategy::ValuationAdaptive)
    .target(Energy::Zero);

let result = optimizer.evolve(topology, rules);

match result {
    VerifiedSuccess(trace) => println!("Evolution successful: {:?}", trace),
    ValidFailure(trace, energy) => println!("Converged to local optima, E={}", energy),
    _ => println!("Evolution failed"),
}
```

---

## Theoretical Foundations
The core engine of Evolver is built upon the following mathematical theories:

1.  **Semi-Tensor Product of Matrices:** Enables operations between matrices of different dimensions, unifying logic and algebra.
2.  **Valuation-Adaptive Perturbation:** An adaptive search strategy based on the geometric features of the energy landscape.
3.  **Topological Dynamics:** Dynamical systems defined on manifolds or graph structures.

For detailed mathematical derivations, please refer to the internal technical documentation in the `theory/` directory.

---

## Copyright & License
**M-Patek PROPRIETARY LICENSE** Copyright © 2025 M-Patek.

This project is **Proprietary Software**. Unauthorized copying, distribution, modification, or commercial use of any part of this software without explicit written permission from the copyright owner is strictly prohibited. This software contains legally protected trade secrets.
