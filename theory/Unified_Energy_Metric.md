# Unified Energy Metric v2.0: The Paraconsistent Hamiltonian

> "The Will climbs the smooth hill; the Truth resides in the sharp valley... but sometimes the valley is underwater."

## 1. Overview

**Revision Note (v2.0):** This document supersedes the previous "Barrier-Residual" model. We have moved from a static penalty model to a dynamic Augmented Lagrangian model to handle logical inconsistencies robustly.

The metric no longer simply measures "Distance to Truth" (which may be infinite in paradoxical systems), but **"Cognitive Dissonance."**

---

## 2. The Objective Function Components

The total Hamiltonian $J$ is composed of three distinct forces:

### 2.1 The Semantic Objective ($E_{obj}$)

Represents the geometric goal of the evolution (the user's intent).

$$E_{obj}(S) = || \Psi_{topo}(S) - \Psi_{target} ||^2$$

**Role:** Guidance. It pulls the system towards the general "shape" of the desired solution.

### 2.2 The Axiomatic Residuals ($C(S)$)

Instead of a binary "True/False", each axiom $i$ produces a continuous residual value $C_i(S) \ge 0$.

* **Syntax Constraints:** $C_{syntax}(S) = 0$ if parser succeeds, else $>0$.
* **STP Constraints:** $C_{stp}(S) = || L(S) - R(S) ||^2$ (Algebraic difference between Left and Right sides of equations).
* **Semantic Axioms:** $C_{axiom}(S) = \text{relu}(\mathbf{X}^T \mathbf{M} \mathbf{X})$ (Penalty for activating mutually exclusive concepts).

### 2.3 The Logical Slack ($\xi$)

A vector variable managed by the optimizer. $\xi_i$ represents the accepted violation of constraint $i$.

---

## 3. The Energy Calculation Logic

In implementation (`src/dsl/stp_bridge.rs`), the energy is no longer a simple `f64`. It is a structured evaluation of the Lagrangian:

$$\mathcal{L}_{\rho}(S, \lambda, \xi) = E_{obj} + \sum (\lambda_i \cdot \Delta_i) + \frac{\rho}{2} \sum \Delta_i^2 + \mu ||\xi||_1$$

Where the effective violation is:

$$\Delta_i = C_i(S) - \xi_i$$

### Interpretation of Energy Levels

| Hamiltonian Value $J$ | System State | Interpretation |
| :--- | :--- | :--- |
| $\approx 0.0$ | **Nirvana** | Perfect Logic. All axioms satisfied ($\xi=0$), Target matched. |
| Low ($< 10.0$) | **Compromise** | Logically valid, or minimal axiom violation ($\xi > 0$). |
| High ($> 100.0$) | **Dissonance** | Major structural conflict. Logic is broken and unrelaxed. |

---

## 4. Verification

The **Proof of Will** now includes the **Compromise Certificate**:

$$\text{Cert} = \{ S_{final}, \xi_{final} \}$$

If $\xi_{final} \neq \vec{0}$, the verifier knows exactly which logic rules were bent to generate the result. This provides transparency rather than silent failure.
