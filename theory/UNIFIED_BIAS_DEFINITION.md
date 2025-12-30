# 统一场论：Bias 向量 $\vec{b}$ 的多重定义与物理实现

## 1. 冲突确认 (The Conflict)

正如敏锐的观察者所指出的，Evolver 项目中关于 Bias 向量 $\vec{b}$ 的定义存在三个维度的视差：

| 来源 | 定义域 | 作用机制 | 视角 |
| :--- | :--- | :--- | :--- |
| THEORY.md | $\vec{b} \in (\mathbb{Z}/L\mathbb{Z})^d$ | 加法群作用：$Out = \Psi(Q) + \vec{b}$ | 密码学视角 (关注满射性和可逆性) |
| Geometry.md | $\vec{b} \in \mathbb{R}^V$ | 向量空间：$Softmax(L + \vec{b})$ | 几何视角 (关注单纯形上的分布移动) |
| Rust Code | $\vec{b}_{raw} \in (\mathbb{Z}/L\mathbb{Z})^{16}$ | 嵌入投影：$L + W_{proj} \cdot \text{Embed}(\vec{b}_{raw})$ | 工程视角 (关注实现的可行性) |

---

## 2. 桥梁：从离散指令到连续干扰 (The Bridge)

为了在工程上实现 THEORY.md 中承诺的“精确控制”，我们实际上构建了一个多级流水线。代码中的实现 (`src/control/bias_channel.rs`) 是这个流水线的完整物理实体。

我们可以将 $\vec{b}$ 重新定义为一个分层对象：

### 第 0 层：控制状态 (Control State) - 对应 THEORY.md

这是 Bias 的本体，是 VAPO 算法直接搜索的对象。

$$\vec{b}_{ctrl} \in (\mathbb{Z}/L\mathbb{Z})^{d_{ctrl}}$$

在代码中，$d_{ctrl} = 16$, $L = 256$ (u8)。
这是离散的，符合群论描述。

### 第 1 层：循环嵌入 (Cyclic Embedding) - 对应 Code

为了让离散的环面坐标能与线性的 Logits 交互，必须将其映射到连续流形。我们使用 sin/cos 编码来保持环面的拓扑结构：

$$\phi: (\mathbb{Z}/L\mathbb{Z}) \to \mathbb{R}^2$$

$$\phi(x) = [\sin(2\pi x/L), \cos(2\pi x/L)]$$

因此，整体嵌入为：

$$\vec{b}_{embed} = \text{Concat}(\phi(b_1), \dots, \phi(b_{16})) \in \mathbb{R}^{32}$$

### 第 2 层：全息投影 (Holographic Projection) - 对应 Optimization.md

Logits 空间的维度 ($V \approx 1024+$) 远大于控制维度 ($32$)。我们使用一个固定的随机投影矩阵 $W_{proj}$ 将控制信号“广播”到整个词表空间：

$$\vec{b}_{logits} = W_{proj} \cdot \vec{b}_{embed} \in \mathbb{R}^V$$

**关键点：** $W_{proj}$ 必须是满秩的（在高维空间通常成立），以保证控制信号不丢失信息。

### 第 3 层：解码融合 (Fusion & Decoding)

最终的输出动作是由 Logits 决定的，而 Logits 是原始信号和 Bias 的叠加：

$$Action = \text{Argmax}(L_{model} + \alpha \cdot \vec{b}_{logits})$$

---

## 3. 为什么 THEORY.md 看起来像是在“撒谎”？

THEORY.md 中的定理 5.7 描述的是：

$$OutCoord = \Psi(Q) + \vec{b}$$

这是一个宏观等效模型 (Effective Model)。

在代码实现中，虽然我们操作的是 Logits，但 VAPO 算法的目标函数是最小化 STP 能量。当 VAPO 找到一个 $\vec{b}_{ctrl}$ 使得 $Energy \approx 0$ 时，实际上就是在寻找一个 $\vec{b}$，使得经过 $\phi$、投影、加法和 Argmax 这一系列复杂的非线性变换后，最终选出的 Token 正好是逻辑所需的那个 Token。

**定理 5.7 在工程上的对应物是：**
只要 $W_{proj}$ 足够随机且 $V$ 足够大，映射 $f(\vec{b}_{ctrl}) = \text{Argmax}(L + W \phi(\vec{b}_{ctrl}))$ 的像集 (Image) 就能够覆盖关键的逻辑路径。

虽然不是数学上的严格满射（Surjective），但在概率意义上，它提供了足够的控制带宽 (Control Bandwidth) 来修正生成器的错误。

---

## 4. 结论与修正计划

我们将保留代码中的实现方式，因为它是在不可微的离散空间（逻辑）和可微的连续空间（神经网络）之间架桥的最佳实践。

**文档修正计划：**

* **THEORY.md** 将保留作为“理想化模型”的地位，描述系统的上限能力。
* **SPECIFICATION.md** 将更新以包含具体的 Embedding 机制描述。
* **Bias_Geometry.md** 将明确区分 $\vec{b}_{ctrl}$ (环面) 和 $\vec{b}_{logits}$ (切空间)。
