# Evolver: The Evolutionary Neural System Architecture
**(Hyper-Tensor Protocol for Neuro-Symbolic Generative AI)**

> "From Probabilistic Guessing to Algebraic Proof."

We are not just eliminating hallucinations; we are constructing the guardrails of thought using number theory.

---

## 📖 Core Vision
Evolver aims to reconstruct the underlying logic of artificial intelligence through the **Hyper-Tensor Protocol (HTP)**. We abandon the black-box probabilistic fitting inherent in the Transformer architecture in favor of a fully interpretable and traceable "**Evolutionary Neural System**."

In this architecture, every inference is not merely the generation of a Token, but a rigorous mathematical proof (**Fiat-Shamir Proof**).

---

## 📐 The Evolution: From Accumulators to Neural Evolution
The core mathematical primitives of Evolver have undergone a qualitative transformation from "membership proofs" to "semantic logical evolution." Below is the evolutionary path of the core formulas:

### 1. The Accumulator Primitive
In early protocol designs (see `HTP.md`), the focus was on processing the temporal accumulation of members ($P_{agent}$):

$$T_{\text{next}} = (T_{\text{prev}}^{P_{\text{agent}}} \cdot G^{H(\text{depth})}) \pmod \Delta$$

**Meaning:** The state $T$ evolves with the addition of agent $P$, and is injected with non-commutative temporal noise via depth $H(\text{depth})$.

### 2. Semantic Evolution
In Phase 3 (Evolutionary Neural System), we reconstructed this formula into a neuronal activation function. Evolution is no longer simple storage, but the non-commutative interaction of semantic weights:

$$S_{out} = S_{in}^{P_{weight}} \cdot G^{H(t)} \pmod \Delta$$

* **$S_{in}$ (Context State):** The algebraic stream of input, carrying the preceding contextual logic.
* **$P_{weight}$ (Semantic Fingerprint):** The "weight" of the neuron. Unlike floating-point weights, this is a massive prime number representing the neuron's specific operation on semantics (e.g., "logical inversion" or "conceptual abstraction").
* **$G^{H(t)}$ (Spacetime Noise):** Injects spacetime depth noise to ensure that "A leads to B" is algebraically distinct from "B leads to A" (non-commutativity).

---

## 🏗️ Phase 3: Evolutionary Neural System Architecture
Based on the latest source code (`src/phase3/structure.rs`, `decoder.rs`), Evolver now possesses full generative capabilities:

### 1. Evolutionary Layer & Neurons
* **RwLock Architecture:** Unlike traditional matrix multiplication, each layer consists of independent `HTPNeuron` units. They process algebraic tuples in parallel and undergo safe structural mutations via `RwLock` during training.
* **Holographic Collapse:** Each neuron maintains a miniature **HyperTensor**. Through a sparse **Fold** algorithm, infinite context is compressed into a unique **Global Root**.

### 2. Inverse Decoder
* **Generation as Navigation:** While Transformers retrieve the most probable word via Softmax, Evolver locates coordinates in algebraic space through **Inverse Decoding**.
* **Spatial Indexing:**
    1.  The model outputs a high-dimensional algebraic root.
    2.  The `InverseDecoder` calculates the corresponding tensor **Coordinate**.
    3.  The **KNN (K-Nearest Neighbors)** algorithm is used to find the nearest legal Token within the `VocabularyTensor`.

### 3. Evolutionary Training
* **Punish Path Mutation:**
    Instead of Backpropagation, we employ evolutionary strategies.
    * **Correct Inference:** The path is preserved (reward).
    * **Incorrect Inference (Hallucination):** Triggers `punish_path_mutation`. The system randomly resets the neuron's prime weights ($P_{weight}$), forcing the network to find a new algebraic path to close the logical loop.

---

## 🧩 Technical Specifications

### Affine Tuple
All computational units are no longer scalars but affine tuples $\mathcal{A} = (P, Q)$, following a non-commutative associative law:

$$\mathcal{A}_1 \oplus \mathcal{A}_2 = (P_1 \cdot P_2, \quad Q_1^{P_2} \cdot Q_2)$$

### Proof-Carrying Code
According to `SPECIFICATION.md`, every output is accompanied by a **ProofBundle** of approximately 280 Bytes:
* **Primary Path:** A Merkle-style path along the challenge axis.
* **Orthogonal Anchors:** Aggregated roots of orthogonal dimensions.
* **Consistency:** Verifies that $\text{Fold}_y(\text{Slice}_y) \equiv \text{GlobalRoot}$.

If verification fails, it indicates the model has produced a "mathematical hallucination," and the output is discarded immediately.

---

## ⚡ Performance Comparison

| Feature | Transformer (Classic) | Evolver (Crystal Brain) |
| :--- | :--- | :--- |
| **Core Logic** | Statistics | Algebraic Evolution |
| **Weight Form** | Float Matrices (Float32) | Large Prime Fingerprints |
| **Context Window** | Limited by $O(N^2)$ Attention | Infinite ($O(\log N)$ Holographic Fold) |
| **Hallucination** | Inherent (Feature) | Mathematical Error (Detected) |
| **Training** | Backpropagation (BP) | Structural Mutation |

---

## 🗺️ Project Status
* [x] **Phase 0: Foundation** (Math Primitives, Class Groups, HTP Core)
* [x] **Phase 1: Topology** (Sparse Hyper-Tensor, Segment Tree Folding)
* [x] **Phase 2: The Probe** (Attention-to-Prime Quantization)
* [x] **Phase 3: Evolutionary Neural System** (Current Focus)
    * [x] `HTPModel` & `CrystalLayer` implementation.
    * [x] `InverseDecoder` and KNN addressing.
    * [x] `EvolutionaryTrainer` mutation logic.
* [ ] Large-scale distributed training tests.

---

## ⚖️ License
**M-Patek PROPRIETARY LICENSE**
Copyright © 2025 M-Patek Research. All Rights Reserved.

*Rebuilding Intelligence, One Prime at a Time.*
