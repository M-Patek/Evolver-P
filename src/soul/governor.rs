// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "The Soul expands when the path narrows."

use std::collections::{HashMap, HashSet};
use rand::Rng;

/// SpectralGovernor (谱隙守护者)
/// 
/// 负责监控当前局部搜索子图的拓扑健康状况。
/// 基于 Expander Graph 理论，健康的 Ramanujan 图应该具有较大的谱隙 (Spectral Gap)。
/// 当谱隙消失 (Gap -> 0) 时，意味着局部几何结构退化为线状或哑铃状，导致混合时间指数级增加。
/// 此时，Governor 会介入并强制进行代数迁移 (Algebra Migration)。
pub struct SpectralGovernor {
    /// 当前定义代数结构的素数 p (决定了 Cayley 图的生成元和规模)
    pub current_p: u64,
    /// 最小允许的谱隙阈值。低于此值视为拓扑坍缩。
    min_spectral_gap: f64,
    /// 历史特征值记录，用于分析收敛趋势或检测周期性坍缩
    gap_history: Vec<f64>,
}

impl SpectralGovernor {
    /// 初始化守护者
    /// initial_p: 初始素数参数
    pub fn new(initial_p: u64) -> Self {
        Self {
            current_p: initial_p,
            min_spectral_gap: 0.05, // 经验阈值：LPS图的渐进界通常远优于此，但在局部子图中0.05已属危险
            gap_history: Vec::new(),
        }
    }

    /// 使用带通缩的幂法 (Deflated Power Iteration) 估算局部算子的第二特征值 lambda_2
    /// 
    /// # 参数
    /// * `states`: 当前局部探索到的节点集合 (Local Cayley Subgraph 的顶点)
    /// * `adjacency`: 局部连接关系 (边)
    /// 
    /// # 返回
    /// * `bool`: 如果谱隙健康返回 true，如果坍缩需要迁移则返回 false
    pub fn check_spectral_gap(&mut self, states: &HashSet<u64>, adjacency: &HashMap<u64, Vec<u64>>) -> bool {
        let n = states.len();
        // 样本太少不具备统计意义，且小图的谱隙通常很大，无需担心
        if n < 20 { 
            return true; 
        } 

        // 1. 初始化随机向量 v 
        // 这一步模拟一个随机分布的初始“热量”或“概率密度”
        let mut rng = rand::thread_rng();
        let mut v: Vec<f64> = (0..n).map(|_| rng.gen::<f64>()).collect();
        
        // 强制中心化：移除主特征向量 (Uniform Distribution) 的分量
        // 因为对于正则图，lambda_1 = 1 对应的特征向量是 [1, 1, ..., 1]
        self.orthogonalize_to_uniform(&mut v);

        // 2. 幂迭代 (Power Iteration)：v_{k+1} = M * v_k
        // LPS 图是 (p+1)-正则图，归一化邻接矩阵 M = A / k
        let k = (self.current_p + 1) as f64; 
        let iterations = 20; // 通常 log(N) 次迭代足以分离出 lambda_2
        
        // 建立 Hash -> Index 的映射以加速稀疏矩阵乘法
        let state_vec: Vec<u64> = states.iter().cloned().collect();
        let state_map: HashMap<u64, usize> = state_vec.iter()
            .enumerate()
            .map(|(i, &s)| (s, i))
            .collect();

        for _ in 0..iterations {
            let mut next_v = vec![0.0; n];
            
            // 执行稀疏矩阵乘法 M * v
            for (idx, state_hash) in state_vec.iter().enumerate() {
                if let Some(neighbors) = adjacency.get(state_hash) {
                    for neighbor in neighbors {
                        if let Some(&neighbor_idx) = state_map.get(neighbor) {
                            // 正常的能量扩散
                            next_v[neighbor_idx] += v[idx] / k;
                        } else {
                            // 边界条件处理 (Dirichlet-like Boundary)：
                            // 邻居不在当前局部子图中。
                            // 在物理上这代表能量耗散到了未探索区域，或者被“弹回”。
                            // 这里我们简化处理：假设能量滞留在当前节点 (Self-loop)，以保持总能量大致守恒，
                            // 避免因为边界效应导致特征值虚假衰减。
                            next_v[idx] += v[idx] / k; 
                        }
                    }
                } else {
                    // 孤立点处理 (虽然在强连通图中不应发生)
                    next_v[idx] += v[idx];
                }
            }
            
            // 重新正交化 (Gram-Schmidt) 
            // 这一步至关重要，防止数值误差导致 v 重新向 lambda_1 (全1向量) 坍缩
            self.orthogonalize_to_uniform(&mut next_v);
            
            // L2 归一化
            let norm_sq: f64 = next_v.iter().map(|x| x * x).sum();
            let norm = norm_sq.sqrt();
            
            if norm < 1e-9 { 
                // 向量消失，说明 lambda_2 非常小（也就是 Gap 很大），这是极好的情况
                return true; 
            } 
            
            v = next_v.iter().map(|x| x / norm).collect();
        }

        // 3. 计算 Rayleigh Quotient 近似 |lambda_2|
        // 由于已经归一化，RQ = v^T * M * v
        // 在收敛后，v 近似于特征向量，所以 lambda_2 ≈ ||v_{new}|| / ||v_{old}||
        // 但我们在循环里做了归一化，所以直接计算 v dot Mv 即可，或者简单地取最后一次迭代的模长变化
        // 这里采用点积法：
        // 注意：这里的 v 已经是最后一次迭代的结果，我们需要再乘一次或者直接用它的自相关性
        // 简化起见，我们假设 v 已经收敛到 lambda_2 的特征空间，且 ||v|| = 1
        // 则 lambda_2 ≈ <v, Mv>。但在上面的循环中 next_v = Mv。
        // 由于我们做了归一化，我们实际上没有保留 Mv 的模长。
        // 让我们用一个简单的估计：最后一次迭代前的模长变化率通常就是特征值。
        // 但既然代码逻辑里每次都归一化了，我们无法直接拿到。
        // 修正：我们重新计算一次 M*v 的模长作为特征值估计。
        
        let lambda_2_est = self.estimate_eigenvalue(&v, &state_vec, &state_map, adjacency, k);
        let gap = 1.0 - lambda_2_est;
        
        self.gap_history.push(gap);
        
        // 可选：记录日志
        // println!("[Governor] Current p={}, Est. Spectral Gap={:.4}", self.current_p, gap);

        gap > self.min_spectral_gap
    }

    /// 辅助：执行一次矩阵乘法并返回模长，用于估计特征值
    fn estimate_eigenvalue(
        &self, 
        v: &Vec<f64>, 
        state_vec: &Vec<u64>, 
        state_map: &HashMap<u64, usize>, 
        adjacency: &HashMap<u64, Vec<u64>>, 
        k: f64
    ) -> f64 {
        let n = v.len();
        let mut next_v = vec![0.0; n];
        
        for (idx, state_hash) in state_vec.iter().enumerate() {
            if let Some(neighbors) = adjacency.get(state_hash) {
                for neighbor in neighbors {
                    if let Some(&neighbor_idx) = state_map.get(neighbor) {
                        next_v[neighbor_idx] += v[idx] / k;
                    } else {
                        next_v[idx] += v[idx] / k; 
                    }
                }
            } else {
                next_v[idx] += v[idx];
            }
        }
        
        // 投影掉主成分
        self.orthogonalize_to_uniform(&mut next_v);
        
        // 返回模长，因为 ||v||=1, ||Mv|| ≈ |lambda_2| * ||v||
        next_v.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// 辅助：使向量正交于全1向量 (即去均值)
    /// 这保证了我们测量的是非平凡特征值 (Non-trivial Eigenvalue)
    fn orthogonalize_to_uniform(&self, v: &mut Vec<f64>) {
        let sum: f64 = v.iter().sum();
        let mean = sum / v.len() as f64;
        for x in v.iter_mut() {
            *x -= mean;
        }
    }

    /// 代数迁移 (Algebra Migration)
    /// 寻找下一个满足条件的素数，扩充状态空间。
    /// 
    /// 策略：
    /// 1. 必须是 p = 1 mod 4 (确保 Gaussian Integers 中的分裂性质，维持图结构)。
    /// 2. 必须有显著的增长 (至少 1.2 倍)，根据 Eichler Mass Formula，类数 h ~ p，
    ///    所以这保证了图的规模显著增大，从而稀释当前的拥堵。
    pub fn migrate_algebra(&mut self) -> u64 {
        let mut candidate = self.current_p + 1;
        loop {
            // 条件 1: p = 1 mod 4
            if candidate % 4 == 1 && is_prime(candidate) {
                // 条件 2: 显著增长 (Growth Factor 1.2)
                if candidate > (self.current_p as f64 * 1.2) as u64 {
                    // println!("[Governor] MIGRATION TRIGGERED. New Algebra p={}", candidate);
                    self.current_p = candidate;
                    
                    // 迁移后清空历史，因为新图的谱性质完全不同
                    self.gap_history.clear();
                    
                    return candidate;
                }
            }
            candidate += 1;
        }
    }
}

/// 基础素数检测
/// 对于当前应用场景下的 p 大小 (通常 < 10000)，试除法效率足够。
fn is_prime(n: u64) -> bool {
    if n <= 1 { return false; }
    if n <= 3 { return true; }
    if n % 2 == 0 || n % 3 == 0 { return false; }
    
    let mut i = 5;
    while i * i <= n {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}
