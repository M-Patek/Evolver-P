// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "To guard the shape of logic, one must count the holes in the void."

use nalgebra::{DMatrix, DVector};
use std::collections::{HashMap, VecDeque};

/// 拓扑签名 (Topological Signature)
/// 描述一个点云集合在特定尺度下的代数拓扑特征。
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalSignature {
    /// Betti-0: 连通分量的数量。
    /// 在逻辑空间中，这代表了状态簇的分离程度。
    pub betti_0: usize,
    
    /// Betti-1: 一维孔洞 (1-dimensional holes / cycles) 的数量。
    /// 在逻辑流形中，非平凡的孔洞通常对应着逻辑死循环、自指结构或复杂的拓扑障碍。
    pub betti_1: usize,
}

/// 同调卫士 (Homological Guard)
/// 
/// 负责监控演化过程中状态云的拓扑结构，防止“语义坍缩” (Semantic Collapse)。
/// 如果生成的逻辑路径在几何上坍缩成一条细线 ($b_1=0$)，或者炸裂成无数碎片 ($b_0 \gg 1$)，
/// 卫士将发出高额的能量惩罚。
pub struct HomologicalGuard {
    /// 连边阈值 (Filtration parameter epsilon)。
    /// 只有距离小于此值的点对才会被视为“连通”，从而构成单纯复形的边。
    epsilon: f64,
}

impl HomologicalGuard {
    /// 初始化卫士
    /// epsilon: 决定了拓扑特征的“分辨率”。过大则所有点连成一片，过小则全是孤立点。
    pub fn new(epsilon: f64) -> Self {
        Self { epsilon }
    }

    /// 计算点云的 Vietoris-Rips 复形的 Betti Numbers
    /// 
    /// 这是一个计算密集型操作，包含：
    /// 1. 构建 1-Skeleton (Graph)
    /// 2. 识别 2-Simplices (Triangles)
    /// 3. 构建边界算子矩阵
    /// 4. 在 GF(2) 域上进行矩阵秩的计算
    pub fn compute_betti_numbers(&self, points: &Vec<DVector<f64>>) -> TopologicalSignature {
        let n = points.len();
        if n == 0 { 
            return TopologicalSignature { betti_0: 0, betti_1: 0 }; 
        }

        // 1. 构建 1-Skeleton (图结构)
        // 邻接矩阵用于快速查询边是否存在
        let mut adj = DMatrix::from_element(n, n, 0u8);
        let mut edges = Vec::new();
        let mut edge_count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                let dist = (&points[i] - &points[j]).norm();
                if dist < self.epsilon {
                    adj[(i, j)] = 1;
                    adj[(j, i)] = 1;
                    edges.push((i, j));
                    edge_count += 1;
                }
            }
        }

        // 2. 计算 Betti-0 (连通分量数)
        // 使用简单的 BFS 或 Union-Find
        let b0 = self.count_connected_components(n, &adj);

        // 3. 识别 2-Simplices (三角形)
        // Vietoris-Rips Complex 定义：如果 (i,j), (j,k), (k,i) 边都存在，则单纯形 {i,j,k} 存在。
        // 这些三角形实质上“填补”了图中的 3-clique 空洞，区分了真正的拓扑孔洞和简单的几何三角。
        let mut triangles = Vec::new();
        // 优化：只遍历已存在的边来寻找三角形，而不是三层循环
        // 这里为了代码清晰度保留三层循环结构，但在稀疏图中应优化
        for i in 0..n {
            for j in (i + 1)..n {
                // 剪枝：如果 (i,j) 不连通，不可能构成包含 i,j 的三角形
                if adj[(i, j)] == 0 { continue; }
                
                for k in (j + 1)..n {
                    if adj[(j, k)] == 1 && adj[(k, i)] == 1 {
                        triangles.push((i, j, k));
                    }
                }
            }
        }

        // 4. 计算 Betti-1
        // 公式: b1 = dim(Ker d_1) - dim(Im d_2)
        // 其中 dim(Ker d_1) 即图的 Cycle Rank = |Edges| - |Vertices| + b0
        let cycle_rank = if edge_count + b0 >= n {
            edge_count - n + b0
        } else {
            0 // 防御性编程，理论上不会发生
        };
        
        if triangles.is_empty() {
             // 如果没有三角形来“填补”任何闭环，那么所有的图循环都是一维孔洞
             return TopologicalSignature { betti_0: b0, betti_1: cycle_rank };
        }

        // 构建边界算子 d_2: C_2 (Triangles) -> C_1 (Edges)
        // 这是一个 m x t 的矩阵，其中 m 是边数，t 是三角形数。
        // 元素为 1 表示该边是该三角形的边界。
        let mut boundary_matrix = DMatrix::from_element(edge_count, triangles.len(), 0u8);
        
        // 建立 Edge -> Index 的快速查找表
        let mut edge_to_idx = HashMap::new();
        for (idx, &(u, v)) in edges.iter().enumerate() {
            edge_to_idx.insert((u, v), idx);
        }

        for (t_idx, &(u, v, w)) in triangles.iter().enumerate() {
            // 一个三角形 {u,v,w} 的边界是三条边：{u,v}, {v,w}, {u,w}
            // 在 Z2 (GF(2)) 域上，系数只能是 0 或 1，加法即异或，减法即加法
            if let Some(&e) = edge_to_idx.get(&(u, v)) { boundary_matrix[(e, t_idx)] = 1; }
            if let Some(&e) = edge_to_idx.get(&(v, w)) { boundary_matrix[(e, t_idx)] = 1; }
            if let Some(&e) = edge_to_idx.get(&(u, w)) { boundary_matrix[(e, t_idx)] = 1; } 
        }

        // 计算边界矩阵的秩 (Rank of Boundary Matrix)
        // 这代表了有多少个循环实际上是三角形的边界（即“被填补的”循环）
        let boundary_rank = self.compute_rank_z2(&mut boundary_matrix);
        
        // 真正的同调维数 = 所有循环 - 边界循环
        let b1 = if cycle_rank >= boundary_rank { cycle_rank - boundary_rank } else { 0 };

        TopologicalSignature { betti_0: b0, betti_1: b1 }
    }

    /// 使用 BFS 计算连通分量数 (Betti-0)
    fn count_connected_components(&self, n: usize, adj: &DMatrix<u8>) -> usize {
        let mut visited = vec![false; n];
        let mut count = 0;
        for i in 0..n {
            if !visited[i] {
                count += 1;
                let mut q = VecDeque::new();
                q.push_back(i);
                visited[i] = true;
                while let Some(u) = q.pop_front() {
                    for v in 0..n {
                        if adj[(u, v)] == 1 && !visited[v] {
                            visited[v] = true;
                            q.push_back(v);
                        }
                    }
                }
            }
        }
        count
    }

    /// 在 GF(2) 有限域上计算矩阵的秩
    /// 使用高斯消元法 (Gaussian Elimination)，但加减法替换为 XOR
    fn compute_rank_z2(&self, mat: &mut DMatrix<u8>) -> usize {
        let (rows, cols) = mat.shape();
        let mut pivot_row = 0;
        
        for col in 0..cols {
            if pivot_row >= rows { break; }
            
            // 1. 寻找主元 (Pivot)
            let mut pivot = None;
            for r in pivot_row..rows {
                if mat[(r, col)] == 1 {
                    pivot = Some(r);
                    break;
                }
            }

            if let Some(r) = pivot {
                // 2. 交换行，将主元移到当前 pivot_row
                if r != pivot_row {
                    for c in 0..cols { // 优化：其实只需要从 col 开始交换
                        let temp = mat[(r, c)];
                        mat[(r, c)] = mat[(pivot_row, c)];
                        mat[(pivot_row, c)] = temp;
                    }
                }

                // 3. 消元
                // 对下方所有该列为 1 的行进行 XOR 操作，使其变为 0
                for i in (pivot_row + 1)..rows {
                    if mat[(i, col)] == 1 {
                        for c in col..cols { // 优化：从 col 开始操作
                            mat[(i, c)] ^= mat[(pivot_row, c)]; 
                        }
                    }
                }
                
                // 主元行下移
                pivot_row += 1;
            }
        }
        // 秩等于主元的数量
        pivot_row
    }

    /// 计算拓扑惩罚
    /// 用于优化器 (Optimizer) 的目标函数中。
    /// 
    /// # 逻辑
    /// 如果当前的局部拓扑结构与目标不一致（例如目标需要闭环而当前没有，或者反之），
    /// 则施加巨大的能量惩罚。这迫使优化器进行“拓扑手术”，而不是仅仅在几何上微调。
    pub fn topology_penalty(&self, sig_current: &TopologicalSignature, sig_target: &TopologicalSignature) -> f64 {
        let mut penalty = 0.0;

        // 惩罚连通性差异 (通常我们希望逻辑是连通的，即 b0 = 1)
        let diff_b0 = (sig_current.betti_0 as i32 - sig_target.betti_0 as i32).abs() as f64;
        penalty += 10.0 * diff_b0;

        // 惩罚循环结构差异 (这是核心：防止语义坍缩或死循环)
        let diff_b1 = (sig_current.betti_1 as i32 - sig_target.betti_1 as i32).abs() as f64;
        
        if diff_b1 > 0.0 {
            // Betti-1 的差异通常意味着根本性的逻辑结构错误，给予极高惩罚
            penalty += 100.0 * diff_b1;
        }

        penalty
    }
}
