# Constraint Semantics: The Rules of Algebra

## 1. The Nature of Constraints

In this architecture, a "Constraint" is no longer an additional rule imposed on an external generator, but rather an intrinsic property of the algebraic structure itself.
Constraints define validity. Only state sequences that satisfy specific algebraic and logical rules can be recognized as "Truth."

## 2. Formal Definition

### 2.1 Algebraic Constraints

These constraints are enforced by the mathematical core (`src/soul/algebra.rs`).

**Discriminant Invariance:**
For any state 

$S_t = (a_t, b_t, c_t)$

on the evolution trajectory, the following must hold:

$$b_t^2 - 4a_t c_t = \Delta$$

**Group Closure:**
The result of any operation 

$S_{new} = S_{old} \circ \epsilon$

must remain within the Ideal Class Group 

$Cl(\Delta)$.

This guarantees that the system never produces "mathematically nonsensical" states.

### 2.2 Logical Constraints

These constraints are verified by the STP engine (`src/dsl/stp_bridge.rs`).

**Type Consistency:**
The action `Define { symbol: "n", type: "Odd" }` must comply with the rules of the type system.

**Causal Consistency:**
The action `Apply { inputs: ["n"] }` requires that the symbol "n" must have been defined in a previous step.

$$a_t \text{ is valid} \iff \text{Preconditions}(a_t) \subseteq \bigcup_{i=0}^{t-1} \text{Effects}(a_i)$$

**Axiomatic Consistency:**
The assertion `Assert { condition }` must evaluate to true under the current STP state.

$$E_{STP}(S_t, a_t) = 0 \iff \text{STP}(S_t) \vdash a_t$$

## 3. Constraint Manifold & Search Dynamics

To resolve the duality between algebraic rigour and heuristic search, we explicitly distinguish between the **Search Space** (where the Will travels) and the **Manifold of Truth** (where the Will aims to arrive).

### 3.1 The Manifold of Truth (Destination)

The Manifold of Truth 

$\mathcal{M}_{truth}$

is the sparse subset of the class group containing seeds that materialize into logically valid paths.

$$\mathcal{M}_{truth} = \{ S \in Cl(\Delta) \mid E_{STP}(\text{Materialize}(S)) = 0 \}$$

This is the target set. A seed 

$S^*$

is considered a valid solution if and only if 

$S^* \in \mathcal{M}_{truth}$.

### 3.2 The Search Space (Journey)

The Search Space is the entire Cayley Graph 

$\mathcal{G}$.

During the optimization process (The Will's Walk), the system traverses states 

$S_{temp}$

that generally do not belong to 

$\mathcal{M}_{truth}$.

$$S_{temp} \in \mathcal{G} \setminus \mathcal{M}_{truth} \implies E_{STP}(S_{temp}) > 0$$

The task of the optimizer (VAPO) is to navigate the graph to converge onto the manifold, minimizing the energy potential 

$J(S)$.

### 3.3 Constraint Classifications

**Hard Constraints (Algebraic Laws):**

* **Definition:** Fundamental laws of the algebraic structure (Group Axioms).
* **Enforcement:** Must be satisfied absolutely at every step of the search.
* **Examples:** Discriminant Invariance ($b^2 - 4ac = \Delta$), Group Membership.
* **Violation:** Results in a System Panic or Invalid State Error. The code simply cannot represent an invalid algebraic state.

**Soft Constraints (Logical Guidance):**

* **Definition:** Semantic requirements of the generated logic.
* **Enforcement:** Can be temporarily violated during the search process. The violation magnitude is quantified as "Energy".
* **Examples:** STP Energy, Logical Consistency.
* **Violation:** Results in a positive energy penalty ($E > 0$). This energy acts as a discrete gradient to guide the VAPO search towards zero.
* **Requirement:** The Final State (The Artifact) must strictly satisfy these constraints ($E=0$).

## 4. Code Mapping

* `ClassGroupElement::compose`: Guarantees algebraic constraints (**Hard**).
* `STPContext::calculate_energy`: Checks logical constraints (**Soft**). If violated, it returns non-zero energy, guiding VAPO to avoid that path.
