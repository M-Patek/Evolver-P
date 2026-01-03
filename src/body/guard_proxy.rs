// Copyright (c) 2025 M-Patek
// Part of the Evolver Project
//
// "Trust, but verify. Especially when the geometry is too good to be true."

use crate::body::topology::{HomologicalGuard, TopologicalSignature};
use nalgebra::DVector;

/// 惰性同调卫士 (Lazy Homology Guard)
/// 
/// 这是一个代理模式 (Proxy Pattern) 的实现，用于保护昂贵的同调计算资源。
/// 只有当候选解满足特定的 "高风险且高诱惑" 条件时，才会触发真正的拓扑检查。
/// 
/// # 核心逻辑
/// 同调检查的时间复杂度通常是 O(N^3) 或更高，不适合在每一步优化中运行。
/// 我们利用两个低成本指标作为 "触发器"：
/// 1. 几何残差 (Geometric Residual): 越低说明越接近目标，诱惑越大。
/// 2. 局部曲率 (Local Curvature): 越低 (越负) 说明空间结构越发散，风险越大。
pub struct LazyGuard {
    /// 内部的真正卫士，负责执行昂贵的代数拓扑计算
    inner: HomologicalGuard,
    
    /// 收敛阈值 (Temptation Threshold)
    /// 当几何误差低于此值时，系统认为可能找到了解，此时值得一查。
    convergence_threshold: f64, 
    
    /// 曲率风险等级 (Risk Threshold)
    /// 当局部 Ricci 曲率低于此值时，说明处于高度双曲或混乱区域，极易产生拓扑孔洞。
    curvature_risk_level: f64,
}

impl LazyGuard {
    /// 初始化惰性卫士
    pub fn new() -> Self {
        Self {
            // epsilon = 0.6 是一个经验值，用于定义连通性的尺度
            inner: HomologicalGuard::new(0.6),
            
            // 几何误差 < 0.1 通常意味着 Sinkhorn Distance 已经非常小
            convergence_threshold: 0.1, 
            
            // Ricci 曲率 < -0.8 意味着极强的发散性 (Tree-like structure)
            curvature_risk_level: -0.8, 
        }
    }

    /// 智能审查逻辑 (Smart Inspection)
    /// 
    /// # 参数
    /// * `candidate_cloud`: 候选状态点云 (投影后的几何坐标)
    /// * `geometric_residual`: 当前状态与目标状态的几何距离 (Sinkhorn Divergence)
    /// * `local_curvature`: 当前所在流形区域的离散 Ricci 曲率
    /// 
    /// # 返回
    /// * `(bool, f64)`: (是否否决, 惩罚值)
    ///   - (false, 0.0): 通过审查 (或跳过审查)
    ///   - (true, penalty): 否决，并返回高额惩罚值
    pub fn inspect(
        &self,
        candidate_cloud: &Vec<DVector<f64>>,
        geometric_residual: f64,
        local_curvature: f64
    ) -> (bool, f64) {
        // Condition A: 诱惑 (Temptation)
        // 几何残差很低，优化器非常自信地认为它找到了真理。
        // 只有在 "看起来是对的" 时候，我们才担心它是 "伪真理"。
        let is_tempting = geometric_residual < self.convergence_threshold;

        // Condition B: 风险 (Risk)
        // 负曲率极高，说明此处地形极其复杂，可能有逻辑死循环、纽结或多义性分支。
        // 在平坦空间 (kappa ~ 0) 或正曲率空间，拓扑通常比较简单，不需要频繁检查。
        let is_risky = local_curvature < self.curvature_risk_level;

        // 惰性触发：只有当 "诱惑" 与 "风险" 叠加时，才启动昂贵的同调检查
        if is_tempting && is_risky {
            // println!("[Guard] SUSPICION DETECTED (Res={:.4}, Kappa={:.2}). Running Homology Check...", geometric_residual, local_curvature);
            
            // [SLOW PATH] 启动真正的代数拓扑计算
            let signature = self.inner.compute_betti_numbers(candidate_cloud);

            // 审查标准：如果你声称是终极真理（收敛点），你不应该包含 1 维孔洞 (Betti-1 > 0)
            // 
            // 哲学解释：
            // Betti-1 > 0 意味着逻辑流形中存在不可收缩的闭环 (Non-contractible Cycle)。
            // 在因果逻辑中，这通常对应于 "循环论证" (Circular Reasoning) 或 "自指悖论"。
            // 真正的真理应当是单连通的 (Simply Connected)，或者是平凡的拓扑结构。
            if signature.betti_1 > 0 {
                // println!("[Guard] VETO: Topological Hole Detected (Betti-1={}). Rejecting Pseudo-Solution.", signature.betti_1);
                
                // 强力否决：返回极高的能量惩罚，迫使 VAPO 优化器立刻逃离该区域
                return (true, 100.0); 
            } else {
                 // 虽然很危险，但经查证拓扑是清白的
                 // println!("[Guard] PASSED: Topology is clean.");
            }
        }

        // [FAST PATH] 默认放行
        // 大多数时候我们走这里，零开销
        (false, 0.0)
    }
}
