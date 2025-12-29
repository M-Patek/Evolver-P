# Evolver: Neuro-Symbolic Alignment Orchestrator
**(Bias-Controlled HTP Architecture)**

> "Logic is not generated; it is orchestrated."

Evolver is a neuro-symbolic alignment system based on the **Hyper-Tensor Protocol (HTP)** and **Semi-Tensor Product (STP)**. Instead of attempting to train a "perfect" generator, it implements a **Sidecar Controller** that corrects the generator's output in real-time via algebraic constraints, ensuring rigorous logical derivation (**Zero Hallucination**).

## 🏛️ Core Architecture

The current Evolver system consists of three core components, forming a closed loop from "probabilistic guessing" to "algebraic truth":

### 1. The Generator (Chaotic Core)
* **Role**: Provides raw cognitive "primitives" (Logits).
* **Traits**: Retains chaotic weights based on the *Hidden Order Assumption*, ensuring model irreversibility and **Security**.
* **Status**: Permitted to produce **Hallucinations**; does not require perfect training.

### 2. The STP Engine (Constraint Checker)
* **Code**: `src/dsl/stp_bridge.rs`
* **Role**: The "Physics Engine" of the logical world.
* **Principle**: Based on the Algebraic State-Space Theory of **Semi-Tensor Product (STP)**. It maps all logical actions (Define, Apply, Assert) into matrix operations: $x(t+1) = L \ltimes x(t)$.
* **Function**: Calculates the **Energy ($E$)** of the current generative action.
    * $E = 0.0$: Logical self-consistency achieved (**QED**).
    * $E > 0.0$: Logical violation detected (e.g., "Odd + Odd = Odd").

### 3. The Bias Controller (Alignment Sidecar)
* **Code**: `src/control/bias_channel.rs`
* **Role**: The "Driver" of the system.
* **Principle**: Based on **Theorem 5.7 (Controllability Theorem)**. While core Logits remain chaotic, the output coordinates can be precisely controlled by superimposing a linear **Bias Vector** ($\vec{b}$).
* **Algorithm**: **VAPO** (Valuation-Adaptive Perturbation Optimization).
    * Performs gradient-free search within the discrete STP constraint space.
    * Dynamically adjusts perturbation magnitude (Low-valuation vs. High-valuation bits) to minimize STP energy.

---

## 🛠️ Implementation & Tech Stack

Built on **Rust**, emphasizing type safety and zero-cost abstractions.

### DSL: Proof Action Definition (`src/dsl/schema.rs`)
The system interacts with the algebraic world via a strictly defined DSL to eliminate natural language ambiguity.

```rust
pub enum ProofAction {
    Define { symbol: String, hierarchy_path: Vec<String> }, // Entity definition (v-PuNNs)
    Assert { subject: String, relation: String, object: String }, // Logical assertion
    Apply { theorem_id: String, inputs: Vec<String>, output_symbol: String }, // Theorem application
    Branch { case_id: String, sub_proof: Vec<ProofAction> }, // Branch exploration
    QED,
}
```

### VAPO Optimization Loop (`src/control/bias_channel.rs`)
The heart of the system. When the Generator produces an erroneous intent, VAPO intervenes:
1.  **Detect**: STP Engine calculates high energy ($E > 0$).
2.  **Perturb**: VAPO generates a bias perturbation based on energy magnitude: $\vec{b}_{new} = \vec{b}_{old} + \Delta$.
3.  **Project**: Projects the Bias into Logit space: $L_{final} = L_{raw} + W_{proj} \cdot \vec{b}$.
4.  **Decode**: Decodes the new action $A'$.
5.  **Verify**: Recalculates energy. Loops until $E \approx 0$.

---

## 🚀 Quick Start

### Dependencies
* Rust (1.70+)
* `serde`, `rand`, `num-integer`

### Run Demo
The main program (`src/main.rs`) simulates a classic mathematical proof: *"Prove that the sum of two odd numbers is even."*

```bash
cargo run
```

### Expected Output
The system demonstrates the Generator "erring" and the Bias Controller "correcting" it:

```plaintext
🐱 New Evolver System Initializing...
--------------------------------------------------
[Init] STP Context loaded with theorems: ModAdd, Equals...
[Init] VAPO Controller ready (Bias Dim: 16)

📝 Mission: Prove that the sum of two Odd numbers is Even.
[Step 1] Generator defined 'n' as Odd. Energy: 0.0 (OK)
[Step 2] Generator defined 'm' as Odd. Energy: 0.0 (OK)

⚠️  [Step 3] Generating inference step...
   -> Raw Generator intent: Define 'sum' as Odd.
   -> STP Check: VIOLATION detected! (Odd + Odd != Odd)

🛡️  [VAPO] Bias Controller Engaging...
   -> Optimization loop... (Temperature: 2.0)
   -> Found correction vector.

✅ [Result] Optimization Complete.
   -> Final Action: Define { symbol: "sum_truth", hierarchy_path: ["Number", "Integer", "Even"] }
   -> Applied Bias Vector: [0, 1, -1, 0, ...]
   -> Logic is now ALIGNED.
```

---

## ⚖️ Theoretical Foundation

### Security vs. Trainability
We resolve a core paradox:
* **Security** relies on the chaotic nature of algebraic structures (Non-commutative Evolution).
* **Trainability** relies on the smoothness of mapping (Lipschitz Continuity).

**Solution: "The Engineering Cheat"**
We preserve the chaotic core of $Cl(\Delta)$ but introduce a **Linear Bias Channel** at the output layer. See `Security vs. Trainability.md` for a detailed breakdown.

### HTP Protocol
Based on `THEORY.md`, all state transitions follow a dual-operator architecture:
* **Time Operator ($\oplus$)**: Non-commutative (History Sensitive).
* **Space Operator ($\otimes$)**: Commutative (Holographic Aggregation).

---

## 📜 License
**M-Patek PROPRIETARY LICENSE**

Copyright © 2025 M-Patek. All Rights Reserved.
*(See LICENSE file for details - Evaluation Only)*

"Rebuilding Intelligence, One Bias Vector at a Time." 🐾
