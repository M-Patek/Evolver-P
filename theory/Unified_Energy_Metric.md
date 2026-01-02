# Unified Energy Metric: The Barrier-Residual Model

"The Will needs a Slope, the Truth needs a Threshold."

## 1. The Definitions

To strictly align the implementation (`src/dsl/stp_bridge.rs`) with the theoretical specification, we define the Total System Energy $J(S)$ as a sum of a Discrete Barrier and a Continuous Residual.

$$J(S) = E_{barrier}(\Psi(S)) + E_{residual}(\Psi(S))$$

### 1.1 The Discrete Barrier ($E_{barrier}$)

The Barrier represents the Logical Validity State. It is a discrete step function derived from the rigor of the STP (Semi-Tensor Product) engine and the Parser.

Values are strictly defined as:

| Value ($B$) | State Meaning | Condition |
| :--- | :--- | :--- |
| **0.0** | TRUTH (Valid) | Parser OK AND Logic OK ($E_{STP} = 0$). |
| **10.0** | SYNTAX ERROR | Parser Failed (e.g., structural mismatch, invalid opcode). |
| **100.0** | LOGICAL FALSE | Parser OK, but Logic Contradiction ($E_{STP} > 0$). |

**Note:** These constants (10.0, 100.0) act as "energy plateaus" that the optimizer seeks to fall off of.

### 1.2 The Continuous Residual ($E_{residual}$)

The Residual represents the Geometric Proximity. It serves as a tie-breaker for invalid states, allowing the VAPO algorithm to perform hill-climbing even when the logic is broken.

$$E_{residual}(S) = \beta \cdot || \mathbf{F}(S) - \mathbf{F}_{target} ||^2$$

Where:
* $\mathbf{F}(S)$ is the Continuous Modular Embedding of state $S$.
* $\beta$ is a small scaling factor (e.g., $0.01$) ensuring $E_{residual}$ never exceeds the gap between Barrier levels.

---

## 2. The Logic of Optimization

The optimizer seeks to minimize $J(S)$. The landscape implies a priority queue of constraints:

1.  **Priority 1: Fix Logic (100.0 -> 0.0)**
    If the system is in a "Logical False" state, the optimizer is driven by the huge energy drop of 100.0 to find any state that is logically valid.
2.  **Priority 2: Fix Syntax (10.0 -> 0.0)**
    If the system is outputting garbage (Syntax Error), the drop of 10.0 encourages finding well-formed structures.
3.  **Priority 3: Optimize Intuition (Residual)**
    If multiple states have the same Barrier level (e.g., both are False), the optimizer chooses the one with the smaller $E_{residual}$. This embodies the principle: "Even if you are wrong, be roughly in the right neighborhood."

---

## 3. Implementation Alignment

The implementation in `src/dsl/stp_bridge.rs` MUST reflect this structure:

```rust
pub fn calculate_energy(state: &IdealClass, target: &FeatureVector) -> f64 {
    let projected_path = project(state);
    
    // 1. Check Syntax
    let ast = match parse(&projected_path) {
        Ok(ast) => ast,
        Err(_) => return 10.0 + geometric_distance(state, target),
    };

    // 2. Check Logic (STP)
    let logic_violation = stp_check(&ast); // Returns true if contradiction found
    
    if logic_violation {
        return 100.0 + geometric_distance(state, target);
    }

    // 3. Truth
    return 0.0; // Strictly 0.0 means Success.
}
```

---

## 4. Summary

* **No Gradients:** The optimization process does not use derivatives. It uses the magnitude of $J(S)$ to greedily select neighbors.
* **Correct-by-Construction:** A result is only accepted if $J(S) < 1.0$ (effectively 0.0), which mathematically guarantees that $E_{barrier} = 0$.
