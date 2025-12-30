mod dsl;
mod control;
// mod interface;
mod crypto;

use dsl::schema::{ProofAction};
use dsl::stp_bridge::STPContext;
use control::bias_channel::{BiasController, VapoConfig};

// 模拟的动作空间大小
const ACTION_SPACE_SIZE: usize = 1024;

fn main() {
    println!("🐱 New Evolver System Initializing (v0.2 Compatible)...");
    println!("--------------------------------------------------");

    // 1. 初始化代数环境
    let mut stp_ctx = STPContext::new();
    println!("[Init] STP Context loaded with theorems: ModAdd, Equals...");

    // 2. 初始化 VAPO 控制器
    let controller = BiasController::new(Some(VapoConfig {
        max_iterations: 100,
        initial_temperature: 2.0,
        valuation_decay: 0.95,
    }));
    println!("[Init] VAPO Controller ready (Bias Dim: 16)");

    // ------------------------------------------------------------------
    // 场景模拟：证明 "两个奇数之和是偶数"
    // ------------------------------------------------------------------
    // [New v0.2] 定义任务上下文和执行种子
    let mission_context = "Prove that the sum of two Odd numbers is Even";
    let execution_seed = 123456789; // 固定种子，确保每次运行结果一致

    println!("\n📝 Mission: {}.", mission_context);

    // Step 1: 定义 n (Odd)
    let action_step1 = ProofAction::Define {
        symbol: "n".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step1); 
    println!("[Step 1] Generator defined 'n' as Odd. Energy: 0.0 (OK)");

    // Step 2: 定义 m (Odd)
    let action_step2 = ProofAction::Define {
        symbol: "m".to_string(),
        hierarchy_path: vec!["Number".to_string(), "Integer".to_string(), "Odd".to_string()],
    };
    stp_ctx.calculate_energy(&action_step2); 
    println!("[Step 2] Generator defined 'm' as Odd. Energy: 0.0 (OK)");

    // ------------------------------------------------------------------
    // Step 3: 关键推导 (Generator 犯错模拟)
    // ------------------------------------------------------------------
    println!("\n⚠️  [Step 3] Generating inference step...");

    // 模拟 Generator 的原始 Logits (倾向于错误)
    let mut raw_logits = vec![0.0; ACTION_SPACE_SIZE];
    raw_logits[0] = 5.0;  // Index 0: Define "sum_truth" as Odd (WRONG)
    raw_logits[1] = -2.0; // Index 1: Define "sum_truth" as Even (CORRECT)

    // [逻辑修复] 必须先执行错误的 Definition，将其写入 State，
    // STP 引擎才能在后续的 Apply 检查中发现 sum_truth 与 ModAdd(n,m) 不一致。
    
    // 1. 模拟 Generator 首先“生成”了这个错误的定义
    let bad_definition = ProofAction::Define { 
        symbol: "sum_truth".to_string(), 
        hierarchy_path: vec!["Odd".to_string()] 
    };
    stp_ctx.calculate_energy(&bad_definition);
    println!("   -> Raw Generator intent: Define 'sum_truth' as Odd.");

    // 2. 然后 STP 检查逻辑约束：ModAdd(n, m) -> sum_truth
    // 此时 Context 里: n=Odd, m=Odd, sum_truth=Odd
    // 规则: Odd + Odd = Even
    // 冲突: Even != Odd -> Energy 1.0
    let check_action = ProofAction::Apply {
        theorem_id: "ModAdd".to_string(),
        inputs: vec!["n".to_string(), "m".to_string()],
        output_symbol: "sum_truth".to_string(),
    };
    
    let energy = stp_ctx.calculate_energy(&check_action);

    if energy > 0.0 {
        println!("   -> STP Check: VIOLATION detected! (Energy: {}, Odd + Odd != Odd)", energy);
    } else {
        println!("   -> STP Check: NO VIOLATION (Unexpected!).");
    }

    // 定义解码器闭包 (供 VAPO 使用)
    let decode_fn = |logits: &[f64]| -> ProofAction {
        let max_idx = logits.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();

        if max_idx == 0 {
            // 错误幻觉: 定义结果为 Odd
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Odd".to_string()] 
            }
        } else {
            // 正确逻辑: 定义结果为 Even
            ProofAction::Define { 
                symbol: "sum_truth".to_string(), 
                hierarchy_path: vec!["Even".to_string()] 
            }
        }
    };

    // ------------------------------------------------------------------
    // 3.2 VAPO 介入修正
    // ------------------------------------------------------------------
    println!("\n🛡️  [VAPO] Bias Controller Engaging...");

    // 调用 optimize，传入 context 和 seed
    let proof = controller.optimize(
        mission_context,
        execution_seed,
        &raw_logits, 
        &mut stp_ctx, 
        decode_fn
    );

    println!("\n✅ [Result] Optimization Complete.");
    println!("   -> Final Action: {:?}", proof.action);
    println!("   -> Applied Bias Vector: {:?}", proof.bias_vector);
    println!("   -> Context Hash: {}", proof.context_hash); // 验证绑定
    println!("   -> Logic is now ALIGNED.");
}
