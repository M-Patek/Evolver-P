# Rigorous Logic Semantics: The Closed Loop

## 1. Abstract

This document provides the formal mathematical definition of the Violation Function and proves the equivalence between the Zero-Energy State and Logical Truth. It bridges the gap between the Algebraic State $S$ and the Energy Potential $E$.

## 2. The Formal Pipeline

The Evolver system is defined by the composition of three functions:

$$E(S) = \mathcal{V} \circ \mathcal{A} \circ \Psi (S)$$

Where:

$\Psi: Cl(\Delta) \to \mathbb{Z}_p^K$ is the Projection Map.

$\mathcal{A}: \mathbb{Z}_p^K \to \text{Action}^*$ is the Semantic Adapter.

$\mathcal{V}: \text{Action}^* \to \mathbb{R}_{\ge 0}$ is the STP Valuation Function.

### 2.1 Matrix Encoding ($\mathcal{E}$)

Every logical symbol $s$ is mapped to a vector $v_s$ in a vector space $V \cong \mathbb{R}^k$.
For a $k$-valued logic system (here $k=2$ for Boolean/Parity logic):

Basis Vectors:

$\text{Even} \cong \delta_2^1 = [1, 0]^T$

$\text{Odd} \cong \delta_2^2 = [0, 1]^T$

### 2.2 Inference Rules as Structure Matrices ($M$)

Every logical inference rule $R$ with $n$ inputs and $1$ output is isomorphic to a Structure Matrix $M_R$ of dimension $k \times k^n$.

Example: Modulo Addition (XOR)
The rule $f(x, y) = (x + y) \pmod 2$ is encoded as:

$$M_{add} = \delta_2 [1, 2, 2, 1] = \begin{bmatrix} 1 & 0 & 0 & 1 \\ 0 & 1 & 1 & 0 \end{bmatrix}$$

This matrix is derived strictly from the truth table of the logic.

## 3. The Violation Function Definition

Let an action $\alpha$ be a tuple $(\text{Rule}, \text{Inputs}, \text{Output})$.
Let $\Sigma$ be the current symbol table (state).

The Physical Truth Vector $v_{phys}$ is calculated via the Semi-Tensor Product:

$$v_{phys} = M_{Rule} \ltimes (\ltimes_{i \in \text{Inputs}} \Sigma(i))$$

The Claimed Vector $v_{claim}$ is the current value of the output symbol:

$$v_{claim} = \Sigma(\text{Output})$$

The Local Energy (Violation) is the Euclidean distance in the vector space:

$$\text{Violation}(\alpha) = \| v_{phys} - v_{claim} \|^2$$

## 4. The Equivalence Proof

**Theorem:** $E_{total} = 0 \iff \text{Logical Consistency}$.

**Proof:**

**Forward ($\Rightarrow$):**
Assume $E_{total} = 0$. Since $E_{total} = \sum \|\cdot\|^2$, every term must be 0.
Therefore, for every inference step:

$$M_{Rule} \ltimes v_{input} = v_{claim}$$

By the fundamental theorem of STP (Cheng, 2011), this algebraic equation holds if and only if the logical relationship defined by $M_{Rule}$ is satisfied by the truth values of inputs and output.
Thus, every step in the sequence is a valid logical deduction.

**Backward ($\Leftarrow$):**
Assume the proof path is logically consistent.
Then for every step, the claimed output matches the result of the logical operator.
Since the Structure Matrix $M$ is the faithful representation of the operator, the matrix multiplication yields the exact vector corresponding to the result.
Thus, $\| v_{phys} - v_{claim} \| = 0$, and $E_{total} = 0$.

**Q.E.D.**

## 5. Conclusion regarding the Optimization Target

The optimization problem:

$$\min_{S \in Cl(\Delta)} E_{STP}(\Psi(S))$$

is now formally equivalent to:
"Find an algebraic state $S$ that projects into a sequence of logical steps which strictly satisfy the matrix equations of the Semi-Tensor Product."

There are no placeholders. The loop is mathematically closed.
