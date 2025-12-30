STATUS: REQUEST FOR COMMENT (RFC)
This document describes the Target Cryptographic Architecture for the HTP Protocol.
The current code implementation (v0.1.0) is a logical prototype using non-cryptographic primitives for demonstration purposes. Features such as Class Groups, Discriminant Generation, and Fiat-Shamir Proof Bundles are NOT currently enforced in the codebase.

# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

## 1. Mathematical Preliminaries

### 1.1 Class Group Parameters

**Discriminant Generation:**
Define $\Delta = -M$, where $M \equiv 3 \pmod 4$ is a prime generated via Hash-to-Prime.

**Security:** Relies on the difficulty of computing the class number $h(\Delta)$.

### 1.2 Dual-Operator System

HTP utilizes two distinct algebraic operators to separate temporal causality from spatial topology.

**Time Operator ($\oplus_{\text{time}}$):** Non-commutative affine composition for history.

$$\mathcal{A}_1 \oplus \mathcal{A}_2 = (P_1 P_2, \quad Q_1^{P_2} Q_2)$$

**Space Operator ($\otimes_{\text{space}}$):** Commutative group aggregation for topology.

$$\mathcal{A}_1 \otimes \mathcal{A}_2 = (P_1 P_2, \quad Q_1 Q_2)$$

## 2. Affine Structure & Optimization

### 2.1 The Affine Tuple

Define the tuple $\mathcal{A} = (P, Q)$ where $P \in \mathbb{Z}$ and $Q \in Cl(\Delta)$.

### 2.2 Time Evolution (Neuron Memory)

Used within HTPNeuron memory cells to record sequential events.

* **Input:** Stream of affine tuples.
* **Aggregation:** Segment Tree using $\oplus_{\text{time}}$.
* **Result:** A single tuple $\mathcal{A}_{\text{cell}}$ representing the entire causal history of that memory cell.

### 2.3 Space Aggregation (Hyper-Tensor)

Used by the HyperTensor to fold dimensions.

* **Input:** Spatial grid of $\mathcal{A}_{\text{cell}}$ (snapshots).
* **Aggregation:** Dimensional folding using $\otimes_{\text{space}}$.
* **Result:** A unique Global Root that is independent of the folding order.

## 3. Hyper-Tensor Topology

### 3.1 Coordinate Mapping

Define the mapping from logical index $i$ to vector $\vec{v}$:

$$v_k = (i // L^{k-1}) \pmod L$$

### 3.2 Dimensional Folding

The tensor dimensionality reduction function $\Phi$ utilizes the Space Operator:

$$\Phi(Tensor_d) = \bigotimes_{i=1}^{L} Tensor_{(i, \dots)}$$

### 3.3 Orthogonal Anchoring

A valid proof for point $\vec{v}$ consists of a hybrid path:

* **Time Path (Micro):** The non-commutative Merkle path inside the cell at $\vec{v}$, verifying the specific event history.
* **Space Path (Macro):** The commutative Merkle path through the tensor dimensions along the Challenge Axis.

**Consistency Check:**
Since $\otimes_{\text{space}}$ is commutative, the Verifier can request folding along any axis (e.g., Y-axis), and the result must match the Global Root.

$$\text{Fold}_{\text{challenge\_axis}}(\text{Slice}) \equiv \text{GlobalRoot}$$

## 4. Protocol Flow

### 4.1 Fiat-Shamir Transformation

Define non-interactive challenge generation:

$$Challenge\_Axis = Hash(Global\_Root \parallel User\_ID) \pmod d$$

### 4.2 Verification Algorithm

Verifier client logic:

1.  **Parse Proof:** Extract Time Path and Space Path.
2.  **Verify Time (Micro):** Recompute the cell's history using $\oplus_{\text{time}}$.
    $$\mathcal{A}_{\text{cell}} = \text{AggregateTime}(\text{TimePath})$$
3.  **Verify Space (Macro):** Aggregate the cell's result with spatial siblings using $\otimes_{\text{space}}$.
    $$\text{ComputedRoot} = \mathcal{A}_{\text{cell}} \otimes \text{AggregateSpace}(\text{SpacePath})$$
4.  **Assert:** Check if $\text{ComputedRoot} == Global\_Root$.
