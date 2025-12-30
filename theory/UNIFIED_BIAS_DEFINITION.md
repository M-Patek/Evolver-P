# 统一场论：Bias 向量 $\vec{b}$ 的多重定义与物理实现

## 1. 冲突确认 (The Conflict)

正如敏锐的观察者所指出的，Evolver 项目中关于 Bias 向量 $\vec{b}$ 的定义存在三个维度的视差。
**特别更新：** 我们已在 `Topological_Closure.md` 中引入了 "Verbalizer" 概念来解决 Logit 空间 ($\mathbb{R}^V$) 与 动作空间 ($A$) 的类型不匹配问题。

| 来源 | 定义域 | 作用机制 | 视角 |
| :--- | :--- | :--- | :--- |
| **THEORY.md** | 
$$\vec{b} \in (\mathbb{Z}/L\mathbb{Z})^d$$
 | 群作用：$Out = \Psi(Q) + \vec{b}$ | 密码学视角 (理想模型) |
| **Topological Patch** | 
$$\vec{b} \in T(\mathbb{R}^V)$$
 | 联络形式：$\sigma' = \sigma + \nu(\vec{b})$ | 拓扑视角 (Verbalized Bundle) |
| **Rust Code** | 
$$\vec{b}_{raw} \in (\mathbb{Z}/L\mathbb{Z})^{16}$$
 | 嵌入投影：$L + W_{proj} \cdot \text{Embed}(\vec{b}_{raw})$ | 工程视角 (物理实现) |

---

## 2. 桥梁：从离散指令到连续干扰 (The Bridge)

为了在工程上实现理论中承诺的“精确控制”，我们构建了一个多级流水线。

### 第 0 层：控制状态 (Control State) - 对应 THEORY.md
这是 VAPO 算法直接搜索的离散对象。

$$\vec{b}_{ctrl} \in (\mathbb{Z}/L\mathbb{Z})^{16}$$

### 第 1 层：循环嵌入 (Cyclic Embedding)
将离散环面坐标映射到连续流形：

$$\phi: (\mathbb{Z}/L\mathbb{Z}) \to \mathbb{R}^2 \implies \vec{b}_{embed} \in \mathbb{R}^{32}$$

### 第 2 层：全息投影 / Verbalizer 近似 (Holographic Projection)
这是关键的修正点。我们需要将控制信号“翻译”成神经网络能听懂的 Token 语言。
在理论上，这是一个 Verbalization Map ($\nu$)。在工程上，我们使用一个固定的随机投影矩阵 $W_{proj}$ 作为 $\nu$ 的线性近似：

$$\vec{b}_{logits} = \nu_{approx}(\vec{b}_{embed}) = W_{proj} \cdot \vec{b}_{embed} \in \mathbb{R}^V$$

**物理意义：** $W_{proj}$ 将抽象的控制意图（如“向左修正”）广播到整个词表空间，以此来模拟对 Token 分布的语义级推力。

### 第 3 层：解码融合 (Fusion & Decoding)
最终的输出动作是由 Logits 决定的。解码器 $\Pi$ 现在被理解为相对于动作嵌入 $\nu(A)$ 的最近邻搜索：

$$Action = \text{Argmax}_{a \in A|_s} \langle L_{model} + \alpha \cdot \vec{b}_{logits}, \nu(a) \rangle$$

---

## 3. 为什么之前看起来像是在“撒谎”？ (The Verbalization Gap)

之前的文档试图直接划等号 $A \cong \mathbb{R}^V$，忽略了 Verbalization Gap。

* **THEORY.md** 描述的是 $\vec{b}$ 对 $A$ 的直接作用（理想控制）。
* **代码实现** 处理的是 $\vec{b}$ 对 $\mathbb{R}^V$ 的物理作用（实际控制）。

**定理 5.7 的工程解释更新：**
只要 $W_{proj}$ (Verbalizer) 足够满秩，且 $\nu(A)$ (动作嵌入) 在 Logit 空间分布得足够稀疏，VAPO 就能找到一个 $\vec{b}$，使得其在 $\mathbb{R}^V$ 中的扰动矢量能够跨越 Voronoi 边界，准确命中目标动作的“吸引子盆地”。

---

## 4. 结论

代码实现 (`src/control/bias_channel.rs`) 保持不变，但其数学解释已获得升级：Bias Channel 本质上是一个逆向的 Verbalizer，它将逻辑约束“翻译”回了 Logit 空间的势能场。
