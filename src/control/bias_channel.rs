use crate::crypto::algebra::{Matrix, Vector};
use rand::Rng;
use std::f64::consts::PI;

/// ASC (Adaptive Subspace Commutation) Projector
/// 
/// 既然静态的低维投影无法覆盖全空间，我们让投影矩阵动态追踪
/// 能量梯度的方向。这就好比一个自动瞄准的炮台。
#[derive(Debug, Clone)]
pub struct BiasProjector {
    /// 投影矩阵 W: [logit_dim x bias_dim]
    /// 将低维控制信号映射到高维 Logit 空间
    projection_matrix: Matrix,
    
    /// 动量缓存，用于平滑旋转，防止投影矩阵剧烈抖动
    momentum: Matrix,
    
    /// 维度配置
    input_dim: usize,  // k (e.g., 16)
    output_dim: usize, // V (e.g., 32000)
    
    /// 学习率/旋转速率
    alpha: f64,
}

impl BiasProjector {
    /// 初始化一个新的投影器，遵循正交性和零均值约束
    /// (Reference: Projection_Constraints.md 2.1 & 2.2)
    pub fn new(input_dim: usize, output_dim: usize) -> Self {
        let mut rng = rand::thread_rng();
        
        // 1. 初始化高斯随机矩阵
        let mut data = vec![vec![0.0; input_dim]; output_dim];
        for i in 0..output_dim {
            for j in 0..input_dim {
                data[i][j] = rng.gen_range(-1.0..1.0);
            }
        }

        let mut proj = BiasProjector {
            projection_matrix: Matrix::new(data),
            momentum: Matrix::zeros(output_dim, input_dim),
            input_dim,
            output_dim,
            alpha: 0.05, // 旋转速率
        };

        // 2. 强制正交化 (Gram-Schmidt) 以满足等距性质 (RIP)
        proj.orthonormalize();
        proj
    }

    /// 核心修复：自适应旋转 (Adaptive Rotation)
    /// 
    /// 当 VAPO 无法将能量降为 0 时，说明最优解在当前的控制子空间之外。
    /// 此函数接收 Logit 空间的能量梯度 (residual)，并将子空间向该梯度方向“倾斜”。
    /// 
    /// Math: W_new = W + alpha * (Residual * b_active^T)
    pub fn rotate_towards_gradient(&mut self, residual: &Vector, active_bias: &Vector) {
        // 1. 计算我们想要的 Logit 修正方向 (Residual)
        // residual 是 [output_dim] 向量
        
        // 2. 计算当前的 Bias 激活状态
        // active_bias 是 [input_dim] 向量
        // 如果 bias 是零，我们不知道旋转哪一列，所以只在 bias 活跃时旋转
        
        // 3. 计算秩-1 更新量 (Rank-1 Update)
        // Update = Residual \otimes Bias^T
        // 这是一个外积，结果是 [output_dim x input_dim] 的矩阵
        
        // 简单的 Hebbian 风格更新：
        // 那些导致了高误差(Residual)的 Bias 分量，其对应的投影列向量应该向 Residual 方向靠拢。
        
        let mut update = Matrix::zeros(self.output_dim, self.input_dim);
        
        // 计算外积并应用动量
        for r in 0..self.output_dim {
            for c in 0..self.input_dim {
                let grad = residual[r] * active_bias[c];
                update[r][c] = grad;
            }
        }
        
        // 4. 应用更新
        // W_{t+1} = W_t - \alpha * \nabla W
        // 这里我们要 *最大化* 覆盖率，所以是让 W 对齐 Residual
        
        for r in 0..self.output_dim {
            for c in 0..self.input_dim {
                self.projection_matrix[r][c] += self.alpha * update[r][c];
            }
        }

        // 5. 重新正交化
        // 这一步至关重要！否则矩阵会坍缩，失去覆盖能力。
        // 我们希望改变方向，但不改变体积。
        self.orthonormalize();
    }

    /// 使用 Gram-Schmidt 过程强制列正交和单位化
    fn orthonormalize(&mut self) {
        let cols = self.input_dim;
        let rows = self.output_dim;
        
        // 提取列向量
        let mut basis: Vec<Vector> = Vec::with_capacity(cols);
        for c in 0..cols {
            let mut col_vec = Vec::with_capacity(rows);
            for r in 0..rows {
                col_vec.push(self.projection_matrix[r][c]);
            }
            basis.push(Vector::new(col_vec));
        }

        // Gram-Schmidt
        for i in 0..cols {
            // 减去在之前基向量上的投影
            for j in 0..i {
                let dot = basis[i].dot(&basis[j]); // assuming basis[j] is already normalized
                let proj = basis[j].scale(dot);
                basis[i] = basis[i].sub(&proj);
            }
            
            // 归一化
            let norm = basis[i].norm();
            if norm > 1e-9 {
                basis[i] = basis[i].scale(1.0 / norm);
            } else {
                // 如果向量坍缩了，重置为随机噪声（防止维度丢失）
                // 这是工程上的 Robustness 处理
                let mut rng = rand::thread_rng();
                let mut new_vec = vec![0.0; rows];
                for k in 0..rows { new_vec[k] = rng.gen_range(-1.0..1.0); }
                let v = Vector::new(new_vec);
                let v_norm = v.norm();
                basis[i] = v.scale(1.0 / v_norm);
            }
        }

        // 写回矩阵，并强制执行 Zero-Mean (Constraint 2.1)
        // 为了满足 Softmax 不变性，每一列的和应该接近 0
        for c in 0..cols {
            let mut sum = 0.0;
            for r in 0..rows { sum += basis[c][r]; }
            let mean = sum / rows as f64;
            
            for r in 0..rows {
                self.projection_matrix[r][c] = basis[c][r] - mean;
            }
        }
    }

    /// 前向投影: Bias(k) -> Logits(V)
    pub fn project(&self, bias: &Vector) -> Vector {
        // y = W * b
        // [V] = [V x k] * [k]
        self.projection_matrix.vector_mul(bias)
    }
}

// ------------------------------------------------------------------
// 辅助 Mock 类型，为了让代码在当前上下文可编译
// 实际项目中这些在 crypto/algebra.rs 中
// ------------------------------------------------------------------
impl Matrix {
    pub fn zeros(rows: usize, cols: usize) -> Self {
        Matrix::new(vec![vec![0.0; cols]; rows])
    }
    
    pub fn vector_mul(&self, vec: &Vector) -> Vector {
        let rows = self.data.len();
        let cols = self.data[0].len();
        assert_eq!(cols, vec.len());
        
        let mut res = Vec::with_capacity(rows);
        for r in 0..rows {
            let mut sum = 0.0;
            for c in 0..cols {
                sum += self.data[r][c] * vec[c];
            }
            res.push(sum);
        }
        Vector::new(res)
    }
}

impl std::ops::Index<usize> for Matrix {
    type Output = Vec<f64>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl std::ops::IndexMut<usize> for Matrix {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}
