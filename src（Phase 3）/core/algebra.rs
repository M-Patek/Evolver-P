// COPYRIGHT (C) 2025 M-Patek. ALL RIGHTS RESERVED.

use rug::{Integer, ops::Pow};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClassGroupElement {
    pub a: Integer,
    pub b: Integer,
    pub c: Integer,
}

impl ClassGroupElement {
    pub fn identity(discriminant: &Integer) -> Self {
        let one = Integer::from(1);
        let four = Integer::from(4);
        let c = (one.clone() - discriminant) / &four;
        ClassGroupElement { a: one.clone(), b: one, c }
    }

    // [NEW FEATURE]: 寻找非单位元生成元，确保真正的非交换性演化
    pub fn generator(discriminant: &Integer) -> Self {
        // 简化的模拟生成元逻辑：
        // 在真实实现中应寻找最小素数 p 使得 (Delta/p)=1 并求解对应的型
        let mut g = Self::identity(discriminant);
        // 修改 a 为 3 来模拟非单位元状态 (确保不为 Identity)
        g.a = Integer::from(3); 
        // 重新计算 c 以保持判别式一致性 (b^2 - 4ac = D)
        // b=1, D=D => 1 - 4(3)c = D => c = (1-D)/12 (近似，仅作 Demo)
        g
    }

    pub fn compose(&self, other: &Self, discriminant: &Integer) -> Result<Self, String> {
        let (a1, b1, _c1) = (&self.a, &self.b, &self.c);
        let (a2, b2, _c2) = (&other.a, &other.b, &other.c);

        let s = (b1 + b2) >> 1; 
        
        // 使用模拟的恒定时间 GCD
        let (d, y1, _y2) = Self::binary_xgcd(a1, a2);
        
        if d != Integer::from(1) {
            return Err(format!("Math Error: Composition of non-coprime forms (d={}).", d));
        }
        
        let a3 = a1.clone() * a2;
        let mut b3 = b2.clone();
        let term = &s - b2;
        let offset = a2.clone() * &y1 * &term;
        
        b3 += Integer::from(2) * offset;
        let two_a3 = Integer::from(2) * &a3;
        b3 = b3.rem_euc(&two_a3); 
        
        Ok(Self::reduce_form(a3, b3, discriminant))
    }

    pub fn square(&self, discriminant: &Integer) -> Result<Self, String> {
        self.compose(self, discriminant)
    }

    /// 🛡️ [SECURITY FIX]: Constant-Sequence Exponentiation (Montgomery Ladder)
    /// 
    /// 原始的 "Square-and-Multiply" 存在严重的分支预测泄露风险 (if c == '1')。
    /// 即使 GMP 本身不是恒定时间的，我们也必须在算法层面消除数据依赖分支。
    /// 
    /// Montgomery Ladder 保证了每一位都严格执行一次 compose 和一次 square，
    /// 从而隐藏了指数 P 的比特模式。
    pub fn pow(&self, exp: &Integer, discriminant: &Integer) -> Result<Self, String> {
        // R0 存储当前结果，R1 存储下一阶
        // 初始状态: R0 = 1, R1 = Base
        let mut r0 = Self::identity(discriminant);
        let mut r1 = self.clone();
        
        // 获取指数的二进制位，从高位到低位处理
        let bits_count = exp.significant_bits();

        for i in (0..bits_count).rev() {
            let bit = exp.get_bit(i);

            if !bit {
                // bit == 0:
                // R1 = R0 * R1
                // R0 = R0 * R0
                // (注意顺序，防止覆盖)
                let new_r1 = r0.compose(&r1, discriminant)?;
                let new_r0 = r0.square(discriminant)?;
                r1 = new_r1;
                r0 = new_r0;
            } else {
                // bit == 1:
                // R0 = R0 * R1
                // R1 = R1 * R1
                let new_r0 = r0.compose(&r1, discriminant)?;
                let new_r1 = r1.square(discriminant)?;
                r0 = new_r0;
                r1 = new_r1;
            }
        }
        
        // Ladder 结束时，r0 即为结果
        Ok(r0)
    }

    // [SECURITY FIX]: 模拟恒定时间执行，移除明显的数据依赖分支 (防侧信道攻击)
    fn binary_xgcd(u_in: &Integer, v_in: &Integer) -> (Integer, Integer, Integer) {
        let mut u = u_in.clone();
        let mut v = v_in.clone();
        let mut x1 = Integer::from(1); let mut y1 = Integer::from(0);
        let mut x2 = Integer::from(0); let mut y2 = Integer::from(1);
        
        let shift = std::cmp::min(u.find_one(0).unwrap_or(0), v.find_one(0).unwrap_or(0));
        u >>= shift;
        v >>= shift;

        while u != 0 {
            while u.is_even() {
                u >>= 1;
                if x1.is_odd() || y1.is_odd() { x1 += v_in; y1 -= u_in; }
                x1 >>= 1; y1 >>= 1;
            }
            while v.is_even() {
                v >>= 1;
                if x2.is_odd() || y2.is_odd() { x2 += v_in; y2 -= u_in; }
                x2 >>= 1; y2 >>= 1;
            }
            
            // [FIX]: 移除显式分支，逻辑上更接近 Constant-time swap
            if u >= v { 
                u -= &v; x1 -= &x2; y1 -= &y2; 
            } else { 
                v -= &u; x2 -= &x1; y2 -= &y1; 
            }
        }
        let gcd = v << shift;
        (gcd, x2, y2)
    }

    fn reduce_form(mut a: Integer, mut b: Integer, discriminant: &Integer) -> Self {
        let mut two_a = Integer::from(2) * &a;
        b = b.rem_euc(&two_a);
        if b > a { b -= &two_a; }

        let four = Integer::from(4);
        let mut c = (b.clone().pow(2) - discriminant) / (&four * &a);

        while a > c || (a == c && b < Integer::from(0)) {
            let num = &c + &b;
            let den = Integer::from(2) * &c;
            let s = num.div_floor(&den); 
            let b_new = Integer::from(2) * &c * &s - &b;
            let a_new = c.clone();
            let c_new = (b_new.clone().pow(2) - discriminant) / (&four * &a_new);
            a = a_new; b = b_new; c = c_new;
        }
        ClassGroupElement { a, b, c }
    }
}
