# Addendum: Theoretical Limits of Projection Control

### 修正：定理 5.7 (Revised) - 子空间内的概率可达性

**原命题的局限性：**
原定理 5.7 假设 $\dim(\vec{b}) = \dim(\mathcal{L})$（Logit 空间维度），从而保证了全空间的满射性。然而，在工程实现中，为了计算效率，我们采用了一个低维嵌入映射 $\Phi: \mathbb{R}^k \to \mathbb{R}^n$，其中 $k \ll n$ (例如 $16 \ll 1024$)。

---

### 修正后的命题 (Subspace Controllability)

定义控制流形 $\mathcal{M}_b = \{ \Phi(\vec{b}) \mid \vec{b} \in \text{Torus}^k \} \subset \mathbb{R}^n$。

系统不再保证对任意目标 Logit $\vec{y}_{target}$ 可达，而是保证在**投影造成的能量损失**可接受的范围内，能够找到最优控制律。

**定义投影残差 (Projection Residual)：**

$$R(\vec{y}) = \min_{\vec{b}} \| \vec{y} - \Phi(\vec{b}) \|$$

**新的控制目标：**
VAPO 算法实际上是在寻找 $\vec{b}^*$，使得：

1. **能量最小化**：$E(\text{decode}(\Phi(\vec{b}^*))) \to 0$
2. **流形约束**：$\Phi(\vec{b}^*)$ 必然落在由随机矩阵 $W$ 定义的低维子空间内。

---

### 工程含义

* **非满射性 (Non-Surjective)**：我们无法强迫生成器输出一个在当前低维投影子空间之外的 token。
* **概率可达 (Probabilistic Reachability)**：依赖于随机投影矩阵 $W$ 满足 **Johnson-Lindenstrauss 引理** 的性质——即高维空间中的逻辑距离在低维投影中得以保持。只要“正确答案”的语义分布不是极其稀疏 (Sparse)，VAPO 就有极大概率能“撞”到一个低能量解。

> **结论更新：**
> Evolver 的安全性保障从“代数上的绝对确定性”降级为“基于流形覆盖率的概率确定性”。为了提高覆盖率，工程上应当定期旋转投影矩阵 $W$ (Matrix Rotation) 或增加 Bias 维度 $k$。
