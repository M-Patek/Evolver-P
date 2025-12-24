# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

## 1. Mathematical Preliminaries

### 1.1 Class Group Parameters
* **Discriminant Generation:**
    Define $\Delta = -M$, where $M$ is the first prime satisfying $M \equiv 3 \pmod 4$ encountered after $Hash(Seed)$.
    > **Principle:** $M \equiv 3 \pmod 4 \implies \Delta \equiv 1 \pmod 4$, which is ideal for generating Odd Class Numbers.
* **Hash-to-Prime:**
    Utilizes a "Nonce-based Hash-and-Test combined with Small Prime Sieve" algorithm.
    * **Input:** Identity
    * **Algorithm:** $SHA256(ID \parallel k) \to \text{Candidate} \to \text{Sieve} \to \text{Miller-Rabin}$.

### 1.2 Non-Commutative Algebra
* **State Evolution:**

$$S_{t} = S_{t-1}^{P_t} \cdot G^{H(t)}$$

* **Recursive Unrolling:**
    Demonstrating the separation of $W$ (Witness) and $R$ (Remainder):

$$T_n = T_{k-1}^{(\prod_{j=k}^n P_j)} \cdot \dots$$

* **Verification Equation:**

$$W^{P_k} \cdot R \equiv T_{root} \pmod \Delta$$

---

## 2. Affine Structure & Optimization

### 2.1 The Affine Tuple
Define the tuple $\mathcal{A} = (P, Q)$ acting on state $S$ as: $S^P \cdot Q$.

### 2.2 Composition Law
Deriving the result of $\mathcal{A}_1$ followed by $\mathcal{A}_2$:

$$\mathcal{A}_{1 \oplus 2} = (P_1 P_2, \quad Q_1^{P_2} \cdot Q_2)$$

> **Note:** Emphasize its Non-commutativity, i.e., $\mathcal{A}_1 \oplus \mathcal{A}_2 \neq \mathcal{A}_2 \oplus \mathcal{A}_1$.

### 2.3 Segment Tree Construction
Describing the tree construction:
* **Leaf:** $\mathcal{A}_i = (P_i, G^{H(i)})$
* **Node:** $\mathcal{A}_L \oplus \mathcal{A}_R$
* **Root:** Represents the aggregated transformation of the entire range.

---

## 3. Hyper-Tensor Topology

### 3.1 Coordinate Mapping
Define the mapping from logical index $i$ to vector $\vec{v}$:

$$v_k = (i // L^{k-1}) \pmod L$$

### 3.2 Dimensional Folding
Define the tensor dimensionality reduction function $\Phi$:

$$\Phi(Tensor_d) \to Tensor_{d-1}$$

Implemented by applying segment tree aggregation across the primary dimension.

### 3.3 Orthogonal Anchoring
Explain the components of a "Proof" for point $\vec{v}$:
1. The main path along the **Challenge Axis**.
2. Roots of orthogonal axes intersecting at $\vec{v}$.
3. **Consistency Check:** $Root(Axis_1) == Root(Axis_2) == GlobalRoot$.

---

## 4. Protocol Flow

### 4.1 Fiat-Shamir Transformation
Define non-interactive challenge generation:

$$Challenge\_Axis = Hash(Global\_Root \parallel User\_ID) \pmod d$$

### 4.2 Verification Algorithm
Verifier client pseudo-code:
1. **Parse Proof:** Parse the proof content.
2. **Recompute Affine Path:** Calculate the aggregated path $\to$ obtain $(P_{agg}, Q_{agg})$.
3. **Compute Result:** $Result = W_{local}^{P_{agg}} \cdot Q_{agg}$.
4. **Assert:** Check if $Result == Global\_Root$.
