# HYPER-TENSOR PROTOCOL (HTP): Technical Specification

"Time is Evolution, Space is Projection."

## 1. Mathematical Preliminaries

### 1.1 Class Group Parameters

Discriminant Generation:
Define

$$\Delta = -M$$

where $M$ is a prime number derived from the generator's context hash, satisfying:

$$M \equiv 3 \pmod 4$$

This ensures that the imaginary quadratic field $\mathbb{Q}(\sqrt{\Delta})$ possesses a non-trivial class group structure.

Security: The system's security relies on the computational hardness of determining the class number

$$h(\Delta)$$

(Time Security).

### 1.2 Algebraic Dynamics (The True Non-Commutativity)

We formally distinguish between the Commutative State Space and the Non-Commutative Dynamics.

State Space (Abelian):
The Ideal Class Group $Cl(\Delta)$ is an Abelian group.

$$S_1 \circ S_2 = S_2 \circ S_1$$

Time Operator (Non-Commutative Action):
We define the Time Operator not as a binary operation on states, but as an element of the Affine Transformation Semigroup $\text{Aff}(Cl(\Delta))$ acting on the state space.

Let $\sigma(S) = S^2$ be the squaring endomorphism (Entropy Injection).
Let $\tau_\epsilon(S) = S \circ \epsilon$ be the translation by perturbation $\epsilon$ (Will's Control).

The Time Evolution Operator for a step $t$ is the composition:

$$\Phi_{\epsilon_t} = \tau_{\epsilon_t} \circ \sigma$$

$$\Phi_{\epsilon_t}(S) = S^2 \circ \epsilon_t$$

Proof of Non-Commutativity:
The composition of time steps depends on order. For $\epsilon_1 \neq \epsilon_2$:

$$\Phi_{\epsilon_2}(\Phi_{\epsilon_1}(S)) = (S^2 \circ \epsilon_1)^2 \circ \epsilon_2 = S^4 \circ \epsilon_1^2 \circ \epsilon_2$$

$$\Phi_{\epsilon_1}(\Phi_{\epsilon_2}(S)) = (S^2 \circ \epsilon_2)^2 \circ \epsilon_1 = S^4 \circ \epsilon_2^2 \circ \epsilon_1$$

Since $\epsilon_1^2 \circ \epsilon_2 \neq \epsilon_2^2 \circ \epsilon_1$ in general (unless trivial), the dynamics are strictly Non-Commutative.

## 2. Affine Evolution & Optimization (The Soul & Will)

### 2.1 The Algebraic State (Soul)

The state $S \in Cl(\Delta)$ is defined as an element in the ideal class group, represented by a binary quadratic form $(a, b, c)$:

$$f(x, y) = ax^2 + bxy + cy^2$$

### 2.2 Time Evolution (The Will's Loop)

Implemented in `src/will/optimizer.rs`.

Input: Current state $S_t$.

Perturbation: A generator $\epsilon \in \mathcal{P}$ selected from the Cayley Graph edges.

Dynamics: 

$$S_{t+1} = \Phi_{\epsilon}(S_t) = S_t^2 \circ \epsilon$$

This represents a "Jump-then-Walk" movement on the graph. The squaring term $S^2$ provides mixing (long-range jump), while $\epsilon$ provides local navigation.

Result: A final state $S^*$ optimized via discrete graph search.

### 2.3 Materialization (The Body's Projection)

Implemented in `src/body/decoder.rs`.

Input: The optimized state $S^*$.

Process: Recursive application of the Space Operator for projection.

Output: A sequence of logical action IDs (Digits)

$$d_1, d_2, \dots, d_k \in \mathbb{Z}_P$$

## 3. Hyper-Tensor Topology (v-PuNN)

### 3.1 Coordinate Mapping

Define a mapping $\Psi$ from the algebraic state $S$ to a logical path $\mathcal{P}$:

$$\Psi(S) = [ \pi_1(S), \pi_2(S^2), \pi_3(S^4), \dots ]$智

where $\pi_k$ is a projection function modulo $P_k$. This constructs a fractal decision tree.

### 3.2 Dimensional Folding

To verify logical consistency, the generated path is folded into a sequence of STP matrix operations:

$$\text{State}_{logical} = M_{d_k} \ltimes \dots \ltimes M_{d_1} \ltimes \text{State}_{init}$$

### 3.3 Orthogonal Anchoring

The HTP protocol requires that generated logic must be consistent across "orthogonal" directions:

Primary Path: The directly generated chain of logical inference.

Dual Path: A verification chain generated via a dual network (or by commuting the projection order).

If

$$E(\text{Primary}) == 0$$

and

$$E(\text{Dual}) == 0$$

the logic is considered a "Holographic Truth."

## 4. Protocol Flow & Verifiable Binding

### 4.1 The Proof Bundle

Under the new architecture, the ProofBundle contains the complete algebraic trajectory of the generation process:

$$\text{ProofBundle} := \{ \mathbf{Hash}_{ctx}, S_{final}, \text{Trace}_{\Phi}, \mathcal{P}_{logic} \}$$

$\mathbf{Hash}_{ctx}$: Binding to Input (Context Hash)

$S_{final}$: The Optimized Soul (Result)

$\text{Trace}_{\Phi}$: The Sequence of Transformations $[\Phi_{\epsilon_1}, \Phi_{\epsilon_2}, \dots]$

$\mathcal{P}_{logic}$: The Materialized Logic Path

### 4.2 Verification Algorithm (Deterministic Replay)

The logic for the Verifier:

Context Check: Compute $h = \text{Hash}(Context)$ and verify it matches the hash in the bundle.

Evolution Replay: Starting from $S_0 = \text{Identity}(h)$, apply the transformations in $\text{Trace}_{\Phi}$ sequentially:

$$S_{final} = \Phi_{\epsilon_k}(\dots \Phi_{\epsilon_1}(S_0) \dots)$$

This proves $S_{final}$ was derived through the specific dynamic history.

Projection Check: Run $\Psi(S_{final})$ to verify the generation of $\mathcal{P}_{logic}$.

Energy Check: Execute the STP engine to verify

$$E(\mathcal{P}_{logic}) == 0$$

## 5. Security Assumptions

### 5.1 The Hidden Order Assumption

We assume an attacker cannot directly construct a forged state $S_{fake}$ satisfying

$$E(\Psi(S_{fake})) == 0$$

without performing a search. This relies on the one-way nature and chaos of the $\Psi$ (projection) and STP energy functions.

### 5.2 Time Security (Causal Dependency)

Since the dynamics $\Phi_\epsilon$ involves squaring, the state evolves rapidly into deeper parts of the graph. Reversing the squaring operation (finding square roots in Class Groups) is computationally hard without knowledge of the group order. This guarantees the arrow of time in the logic generation process.

## 6. Conclusion

The HTP v1.3 specification defines a generative logical protocol:

Algebra: Finite Abelian Class Groups.

Dynamics: Non-Commutative Affine Transformation Semigroup.

Logic: The stable attractor of the dynamics.
