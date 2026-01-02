# Unified Energy Metric: The Barrier-Residual Model

"The Will needs a Slope, the Truth needs a Threshold."

## 1. The Definitions

To strictly align the implementation (`src/dsl/stp_bridge.rs`) with the theoretical specification, we define the Total System Energy $J(S)$ as a sum of a Discrete Barrier, a Semantic Axiom Penalty, and a Continuous Residual.

$$J(S) = E_{barrier}(\Psi(S)) + E_{axiom}(\Psi(S)) + E_{residual}(\Psi(S))$$

### 1.1 The Discrete Barrier ($E_{barrier}$)

The Barrier represents the Syntactic and Structural Validity.

| Value ($B$) | State Meaning | Condition |
| :--- | :--- | :--- |
| 0.0 | STRUCTURALLY OK | Parser OK AND Dimensions Match. |
| 10.0 | SYNTAX ERROR | Parser Failed (e.g., structural mismatch, invalid opcode). |
| 100.0 | LOGICAL FALSE | STP Logic Contradiction ($E_{STP} > 0$). |

### 1.2 The Semantic Axiom Penalty ($E_{axiom}$) [NEW]

To prevent "Consistent but Absurd" states (e.g., proving $5$ is Even), we inject static Axiom Matrices.

$$E_{axiom} = \mathbf{X}^T \mathbf{M}_{axiom} \mathbf{X}$$

Where $\mathbf{M}_{axiom}$ encodes penalties for violating fundamental truths (e.g., Mutual Exclusion of Even/Odd).

* **Logic:** If the system attempts to activate conflicting semantic concepts simultaneously, this term explodes (e.g., $+10,000$).
* **Role:** Acts as the "Common Sense" filter. It ensures that semantic meaning is preserved even if type signatures match.

### 1.3 The Continuous Residual ($E_{residual}$)

The Residual represents the Geometric Proximity. It serves as a tie-breaker for invalid states.

$$E_{residual}(S) = \beta \cdot || \mathbf{F}(S) - \mathbf{F}_{target} ||^2$$

---

## 2. The Logic of Optimization

The optimizer seeks to minimize $J(S)$. The landscape implies a priority queue of constraints:

1.  **Priority 1: Enforce Axioms ($E_{axiom}$)**
    The system must not violate fundamental math. A "structurally valid" proof that $5$ is Even is worse than a syntax error. It is a lie.
2.  **Priority 2: Fix Logic ($E_{barrier}$)**
    Resolve STP structural contradictions.
3.  **Priority 3: Fix Syntax ($E_{barrier}$)**
    Fix parser errors.
4.  **Priority 4: Optimize Intuition ($E_{residual}$)**
    Hill-climb towards the geometric target.

---

## 3. Implementation Alignment

The implementation in `src/dsl/stp_bridge.rs` reflect this structure:

```rust
pub fn calculate_energy(state: &IdealClass, target: &FeatureVector) -> f64 {
    // 1. Syntax Check
    let ast = match parse(project(state)) {
        Ok(ast) => ast,
        Err(_) => return 10.0 + dist,
    };

    // 2. Axiom Check (The Truth Police)
    let axiom_penalty = calculate_axiom_penalty(&ast);
    if axiom_penalty > 0.0 {
        return 100.0 + axiom_penalty; // Huge penalty for lying
    }

    // 3. Structure Check
    let struct_violation = stp_check(&ast);
    if struct_violation {
        return 100.0;
    }

    return 0.0; // Success
}
```
