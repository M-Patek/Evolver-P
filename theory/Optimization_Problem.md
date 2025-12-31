# Optimization Problem: Algebraic State Search

## 1. The Paradigm Shift

In the architecture, the optimization problem is no longer about finding a "bias vector ($\vec{b}$)" in continuous logit space to correct a probability distribution. Instead, it is about searching for an "Ideal State ($S$)" on a discrete algebraic manifold to generate truth.

Optimization is no longer "Correction," but "Discovery."

---

## 2. Problem Formulation

### 2.1 Decision Variable

The optimization object is an element of the ideal class group $Cl(\Delta)$ of an imaginary quadratic field $\mathbb{Q}(\sqrt{\Delta})$.

$$
S = (a, b, c) \in Cl(\Delta)
$$

Where $S$ satisfies the discriminant constraint: 

$$
b^2 - 4ac = \Delta
$$

In the codebase, this corresponds to the `ClassGroupElement` struct in `src/soul/algebra.rs`.

### 2.2 Objective Function

We seek a state $S^*$ such that the logical path generated through its topological projection satisfies the zero-energy constraint of the STP (Semi-Tensor Product) engine.

$$
\text{Minimize } J(S) = E_{STP}(\Psi(S))
$$

Where:

* $\Psi(S)$: Materialization Map. It projects the algebraic state $S$ into a discrete sequence of actions (Digits/Actions).
    * Implementation: `src/body/decoder.rs -> materialize_path`
* $E_{STP}$: Energy Evaluation. Checks whether the action sequence violates logical rules.
    * Implementation: `src/dsl/stp_bridge.rs -> calculate_energy`

### 2.3 Constraints

* **Algebraic Closure**: $S$ must always remain within the group $Cl(\Delta)$. This constraint is automatically satisfied by the mathematical properties of the group operation (compose).
* **Computational Budget**: Number of iterations 

$$
N_{iter} \le N_{max}
$$

---

## 3. Search Space Structure

The search space is not a Euclidean space $\mathbb{R}^n$, but a **Cayley Graph**.
Nodes are group elements; edges are generators (perturbations).

### 3.1 Neighborhood Definition

For the current state $S_{curr}$, its neighborhood $\mathcal{N}(S_{curr})$ is defined as:

$$
\mathcal{N}(S_{curr}) = \{ S_{curr} \circ \epsilon \mid \epsilon \in \mathcal{P} \}
$$

Where $\mathcal{P}$ is the **Perturbation Set**, consisting of small prime ideals with specific norms.
Implementation: `src/will/perturber.rs -> generate_perturbations`

### 3.2 Valuation-Adaptive

The "topography" of the search space is determined by $p$-adic valuation.

* **High Valuation (Small Norm)**: Corresponds to short edges in the group graph, used for local fine-tuning.
* **Low Valuation (Large Norm)**: Corresponds to long jumps (tunneling) in the group graph, used to escape local minima.

---

## 4. Solver Algorithm: VAPO (Valuation-Adaptive Perturbation Optimization)

VAPO is a variant of the Discrete Metropolis-Hastings Search running on a group manifold.

### Algorithm Flow:

1.  **Initialization**: 
    $$S_0 = \text{Hash}(Context)$$
2.  **Iteration** ($t=1 \dots N$):
    * **Schedule**: Calculate the "perturbation window" based on current progress. Initial stages allow large jumps (using large primes); later stages shrink to fine adjustments (using only small primes).
    * **Propose**: Generate a set of candidate states 
        $$\{ S_i' = S_t \circ \epsilon_i \}$$
    * **Evaluate**: Parallelly calculate the STP energy for each candidate 
        $$E_i = J(S_i')$$
    * **Select**:
        * If $\min(E_i) = 0$, Bingo! Return $S_{best}$ immediately.
        * Otherwise, greedily move to the state with the lowest energy 
            $$S_{t+1} = \text{argmin}(S_i')$$
    * *(Optional)* Introduce a Simulated Annealing mechanism to allow acceptance of inferior solutions with a certain probability (current code is purely greedy).

---

## 5. Conclusion

This optimization problem is a **Black-box Discrete Optimization Problem**.

We do not calculate gradients $\nabla J$ (since $J$ is a step function and $S$ is discrete); instead, we exploit the **Orbit Structure** of the algebraic group to traverse the state space.

The optimizer's task is to "spin" on the group orbit until it hits the truth.
