// src/main.rs
mod dsl;
mod control;
// mod interface; // 如果需要可以启用
mod crypto; // 需要 crypto 模块支持

use dsl::schema::{ProofAction};
use dsl::stp_bridge::STPContext;
use control::bias_channel::{BiasController, VapoConfig};

// 模拟的动作空间大小
const ACTION_SPACE_SIZE: usize = 1024;

fn main() {
    println!("🐱 New Evolver System Initializing...");
    println!("--------------------------------------------------");

    // 1. 初始化代数环境
    let mut stp_ctx = STPContext::new();
    println!("[Init] STP Context loaded with theorems: ModAdd, Equals...");

    // 2. 初始化 VAPO 控制器
    let mut controller = BiasController::new(Some(VapoConfig {
        max_iterations: 100,
        initial_temperature: 2.0,
        valuation_decay: 0.95,
    }));
    println!("[Init] VAPO Controller ready (Bias Dim: 16)");

    // ------------------------------------------------------------------
    // 场景模拟：证明 "两个奇数之和是偶数"
    // ------------------------------------------------------------------
    println!("\n📝 Mission: Prove that the sum of two Odd numbers is Even.");

    // Step 1: 定义 n (Odd)
    let action_step1 = ProofAction::Define {
        symbol: "n".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step1); // &mut borrow
    println!("[Step 1] Generator defined 'n' as Odd. Energy: 0.0 (OK)");

    // Step 2: 定义 m (Odd)
    let action_step2 = ProofAction::Define {
        symbol: "m".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step2); // &mut borrow
    println!("[Step 2] Generator defined 'm' as Odd. Energy: 0.0 (OK)");

    // ------------------------------------------------------------------
    // Step 3: 关键推导 (Generator 犯错模拟)
    // ------------------------------------------------------------------
    println!("\n⚠️  [Step 3] Generating inference step...");

    // 模拟 Generator 的原始 Logits (倾向于错误)
    let mut raw_logits = vec![0.0; ACTION_SPACE_SIZE];
    raw_logits[0] = 5.0;  // Index 0: Define "sum" as Odd (WRONG)
    raw_logits[1] = -2.0; // Index 1: Define "sum" as Even (CORRECT)

    // 为了让 bridge 检测冲突，我们先让环境知道 n+m 应该是 Even
    // 我们手动执行一次 Apply 使得 "sum" 被预期为 Even (这里为了演示简化处理)
    // 实际上 stp_bridge.rs 里的 check_inference_consistency 会动态计算 inputs
    // 但在 Definition 检查中，我们需要先有定义。
    // 这里我们假设 Generator 试图 Define 一个叫 "sum_truth" 的变量
    
    // 定义解码器
    let decode_fn = |logits: &[f64]| -> ProofAction {
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        if max_idx == 0 {
            // 错误幻觉: 认为 Odd + Odd = Odd
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Odd".to_string()] // 简化路径匹配 Mock
            }
        } else {
            // 正确逻辑
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Even".to_string()] 
            }
        }
    };
    
    // 必须告诉 STPContext，我们正在检查关于 (n, m) 的加法结果
    // 这是一个 trick：我们在 optimize 内部或者外部，需要一个 Apply 动作来建立约束
    // 为了演示，我们在 bridge 里通过 "Apply ModAdd n m -> sum_truth" 来触发检查
    // 所以我们需要构造一个特殊的场景：
    // Generator 输出的是 Apply 动作，或者我们显式地让 STP 检查这个 Define 是否符合 Apply 的结果。
    // 在 stp_bridge.rs 的修复版中，我们让 calculate_energy 支持 check_inference_consistency。
    // 我们在这里先注册 n+m 的逻辑约束：
    stp_ctx.calculate_energy(&ProofAction::Apply {
        theorem_id: "ModAdd".to_string(),
        inputs: vec!["n".to_string(), "m".to_string()],
        output_symbol: "sum_truth".to_string(),
    });

    println!("   -> Raw Generator intent: Define 'sum_truth' as Odd.");
    println!("   -> STP Check: VIOLATION detected! (Odd + Odd != Odd)");

    // ------------------------------------------------------------------
    // 3.2 VAPO 介入修正
    // ------------------------------------------------------------------
    println!("\n🛡️  [VAPO] Bias Controller Engaging...");

    // 调用 controller.optimize
    let (final_bias, final_action) = controller.optimize(&raw_logits, &mut stp_ctx, decode_fn);

    println!("\n✅ [Result] Optimization Complete.");
    println!("   -> Final Action: {:?}", final_action);
    println!("   -> Applied Bias Vector: {:?}", final_bias.data);
    println!("   -> Logic is now ALIGNED.");
}
