use std::ops::{Add, Mul};
use std::fmt;

/// 严格的半张量积 (STP) 数学内核
/// 
/// 实现了基于 LCM (最小公倍数) 的维度扩充算法。
/// 旨在解决 "Mock Chaos" 原型中维度处理不严谨的问题。
/// 
/// Reference: "Semi-Tensor Product of Matrices", Daizhan Cheng.

#[derive(Clone, Debug, PartialEq)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f64>, // 使用一维数组展平存储，行优先
}

impl Matrix {
    /// 创建新矩阵
    pub fn new(rows: usize, cols: usize, data: Vec<f64>) -> Self {
        assert_eq!(data.len(), rows * cols, "Data length must match dimensions");
        Matrix { rows, cols, data }
    }

    /// 创建单位矩阵 I_n
    pub fn identity(n: usize) -> Self {
        let mut data = vec![0.0; n * n];
        for i in 0..n {
            data[i * n + i] = 1.0;
        }
        Matrix { rows: n, cols: n, data }
    }

    /// 获取元素 (i, j)
    pub fn get(&self, row: usize, col: usize) -> f64 {
        self.data[row * self.cols + col]
    }

    /// Kronecker Product (张量积)
    /// A (x) B
    pub fn kron(&self, other: &Matrix) -> Matrix {
        let new_rows = self.rows * other.rows;
        let new_cols = self.cols * other.cols;
        let mut new_data = vec![0.0; new_rows * new_cols];

        for i in 0..self.rows {
            for j in 0..self.cols {
                let val_a = self.get(i, j);
                // 优化：如果 A 的元素是 0，可以跳过这一块的乘法
                if val_a.abs() < 1e-10 { continue; }

                for k in 0..other.rows {
                    for l in 0..other.cols {
                        let val_b = other.get(k, l);
                        let target_row = i * other.rows + k;
                        let target_col = j * other.cols + l;
                        new_data[target_row * new_cols + target_col] = val_a * val_b;
                    }
                }
            }
        }

        Matrix::new(new_rows, new_cols, new_data)
    }

    /// 标准矩阵乘法 (MatMul)
    /// 要求 self.cols == other.rows
    pub fn matmul(&self, other: &Matrix) -> Result<Matrix, String> {
        if self.cols != other.rows {
            return Err(format!(
                "Dimension mismatch for standard MatMul: ({}, {}) vs ({}, {})",
                self.rows, self.cols, other.rows, other.cols
            ));
        }

        let new_rows = self.rows;
        let new_cols = other.cols;
        let common_dim = self.cols;
        let mut new_data = vec![0.0; new_rows * new_cols];

        for i in 0..new_rows {
            for k in 0..common_dim {
                let val_a = self.get(i, k);
                if val_a.abs() < 1e-10 { continue; } // 稀疏优化

                for j in 0..new_cols {
                    let val_b = other.get(k, j);
                    new_data[i * new_cols + j] += val_a * val_b;
                }
            }
        }

        Matrix::new(new_rows, new_cols, new_data)
    }

    /// 半张量积 (Semi-Tensor Product)
    /// A |x| B = (A (x) I_n) * (B (x) I_p)
    /// 自动处理维度扩充
    pub fn stp(&self, other: &Matrix) -> Matrix {
        let n = self.cols;
        let p = other.rows;

        // 1. 计算最小公倍数 LCM
        let t = lcm(n, p);

        // 2. 计算扩充因子
        let alpha = t / n; // A 需要扩充的倍数
        let beta = t / p;  // B 需要扩充的倍数

        // 3. 构建单位矩阵
        let i_alpha = Matrix::identity(alpha);
        let i_beta = Matrix::identity(beta);

        // 4. 执行 Kronecker 积扩充
        // 左操作数 A 扩充: A (x) I_alpha
        let a_expanded = self.kron(&i_alpha);
        
        // 右操作数 B 扩充: B (x) I_beta
        let b_expanded = other.kron(&i_beta);

        // 5. 执行标准矩阵乘法
        // 此时 a_expanded 的列数应为 n * alpha = t
        // b_expanded 的行数应为 p * beta = t
        // 维度匹配，可以相乘
        a_expanded.matmul(&b_expanded).expect("STP Logic Error: Dimensions should align after expansion")
    }
}

// 辅助函数：最大公约数
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

// 辅助函数：最小公倍数
fn lcm(a: usize, b: usize) -> usize {
    if a == 0 || b == 0 {
        0
    } else {
        (a * b) / gcd(a, b)
    }
}

// 单元测试：验证 STP 的标准性质
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stp_dimension_matching() {
        // Case 1: 向量 (1x2) 乘 矩阵 (4x2)
        // 这里的 STP 允许行向量乘以列数不匹配的矩阵，只要满足倍数关系
        // X (1x2), Y (4x2). Cols(X)=2, Rows(Y)=4. LCM(2,4)=4.
        // X 扩充 2倍 -> 1x4. Y 扩充 1倍 -> 4x2.
        // 结果应为 (1x2)
        
        let x = Matrix::new(1, 2, vec![1.0, 2.0]);
        let y = Matrix::new(4, 2, vec![
            1.0, 0.0,
            0.0, 1.0,
            1.0, 0.0,
            0.0, 1.0
        ]);

        let result = x.stp(&y);
        
        // X (x) I_2 = [1, 0, 2, 0; 0, 1, 0, 2] (Error logic check manually if complex)
        // 正确推导:
        // X (x) I_2 = [1, 2] (x) [1,0; 0,1] 
        // = [1*1, 1*0, 2*1, 2*0; 1*0, 1*1, 2*0, 2*1] -> 这变成了 2x4 矩阵? 
        // 等等，Kronecker 定义:
        // A (m x n), B (p x q) -> mp x nq
        // X (1x2), I (2x2) -> 2x4.
        // [1*[1,0;0,1], 2*[1,0;0,1]] = [1, 0, 2, 0; 0, 1, 0, 2].
        // 
        // Y (4x2), I (1x1) -> Y.
        // Result = (2x4) * (4x2) -> 2x2.
        
        println!("Result shape: {}x{}", result.rows, result.cols);
        assert_eq!(result.rows, 2);
        assert_eq!(result.cols, 2);
    }

    #[test]
    fn test_stp_degenerates_to_matmul() {
        // 当维度匹配时，STP 应当等于 MatMul
        let a = Matrix::new(2, 2, vec![1.0, 0.0, 0.0, 1.0]); // I
        let b = Matrix::new(2, 2, vec![1.0, 2.0, 3.0, 4.0]); // B

        let res_stp = a.stp(&b);
        let res_mul = a.matmul(&b).unwrap();

        assert_eq!(res_stp.data, res_mul.data);
    }
}
