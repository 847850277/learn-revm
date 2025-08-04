use stage2_architecture::*;

/// 练习 1: 规范系统深入理解
///
/// 本练习将展示：
/// 1. 如何定义和实现 EVM 规范
/// 2. 编译时常量的性能优势
/// 3. 不同硬分叉的差异对比
/// 4. 规范驱动的设计模式

fn main() {
    println!("🎯 练习 1: EVM 规范系统深入理解");
    println!("{}", "=".repeat(60));

    // 演示 1: 规范常量的编译时绑定
    demonstrate_compile_time_binding();

    // 演示 2: 不同规范的对比分析
    demonstrate_spec_comparison();

    // 演示 3: 规范特性的条件编译
    demonstrate_conditional_features();

    // 演示 4: Gas 成本计算差异
    demonstrate_gas_differences();

    // 演示 5: 预编译合约支持
    demonstrate_precompile_support();

    println!("\n🎉 练习 1 完成！您已经理解了 EVM 规范系统的核心设计理念。");
}

/// 演示编译时常量绑定的优势
fn demonstrate_compile_time_binding() {
    println!("\n📊 演示 1: 编译时常量绑定");
    println!("{}", "-".repeat(40));

    // 使用不同规范的常量
    println!("各规范的 CALL 指令 Gas 成本:");
    println!("  Frontier: {} gas", spec::Frontier::GAS_CALL);
    println!("  Berlin:   {} gas", spec::Berlin::GAS_CALL);
    println!("  London:   {} gas", spec::London::GAS_CALL);

    println!("\n💡 关键优势:");
    println!("  ✅ 零运行时成本 - 所有值在编译时确定");
    println!("  ✅ 类型安全 - 不同规范无法混用");
    println!("  ✅ 内联优化 - 编译器可以直接内联常量");

    // 演示泛型函数如何使用规范常量
    fn calculate_call_cost<S: Spec>() -> u64 {
        // 这个函数会为每个规范生成专门的版本
        S::GAS_CALL + 100 // 假设额外成本
    }

    println!("\n📈 泛型函数中的规范使用:");
    println!(
        "  Frontier CALL 总成本: {}",
        calculate_call_cost::<spec::Frontier>()
    );
    println!(
        "  Berlin CALL 总成本:   {}",
        calculate_call_cost::<spec::Berlin>()
    );
    println!(
        "  London CALL 总成本:   {}",
        calculate_call_cost::<spec::London>()
    );
}

/// 演示不同规范的详细对比
fn demonstrate_spec_comparison() {
    println!("\n🔍 演示 2: 规范对比分析");
    println!("{}", "-".repeat(40));

    // 比较 Frontier vs Berlin
    println!("📊 Frontier vs Berlin Gas 成本变化:");
    let gas_changes = spec::SpecComparison::compare_gas_costs::<spec::Frontier, spec::Berlin>();
    for (operation, old_cost, new_cost, diff) in gas_changes {
        let change_indicator = if diff > 0 {
            "📈 +"
        } else if diff < 0 {
            "📉 "
        } else {
            "➡️  "
        };
        println!(
            "  {}: {} -> {} {} {}",
            operation, old_cost, new_cost, change_indicator, diff
        );
    }

    // 比较特性支持
    println!("\n🔧 Frontier vs Berlin 特性支持:");
    let feature_changes = spec::SpecComparison::compare_features::<spec::Frontier, spec::Berlin>();
    for (feature, old_support, new_support) in feature_changes {
        let change = match (old_support, new_support) {
            (false, true) => "🆕 新增",
            (true, false) => "🗑️ 移除",
            (true, true) => "✅ 保持",
            (false, false) => "❌ 不支持",
        };
        println!("  {}: {}", feature, change);
    }

    // 比较 Berlin vs London
    println!("\n📊 Berlin vs London 特性演进:");
    let london_features = spec::SpecComparison::compare_features::<spec::Berlin, spec::London>();
    for (feature, berlin_support, london_support) in london_features {
        if berlin_support != london_support {
            println!(
                "  {}: {} -> {}",
                feature,
                if berlin_support { "✅" } else { "❌" },
                if london_support { "✅" } else { "❌" }
            );
        }
    }
}

/// 演示条件特性编译
fn demonstrate_conditional_features() {
    println!("\n⚙️ 演示 3: 条件特性编译");
    println!("{}", "-".repeat(40));

    // 模拟特性检查函数
    fn check_create2_support<S: Spec>() -> &'static str {
        if S::ENABLE_CREATE2 {
            "支持 CREATE2 - 可确定性部署"
        } else {
            "不支持 CREATE2 - 仅支持传统 CREATE"
        }
    }

    fn check_eip1559_support<S: Spec>() -> &'static str {
        if S::ENABLE_EIP1559 {
            "支持 EIP-1559 - 新手续费机制"
        } else {
            "传统手续费机制"
        }
    }

    println!("🔧 CREATE2 支持检查:");
    println!("  Frontier: {}", check_create2_support::<spec::Frontier>());
    println!("  Berlin:   {}", check_create2_support::<spec::Berlin>());
    println!("  London:   {}", check_create2_support::<spec::London>());

    println!("\n💰 EIP-1559 支持检查:");
    println!("  Frontier: {}", check_eip1559_support::<spec::Frontier>());
    println!("  Berlin:   {}", check_eip1559_support::<spec::Berlin>());
    println!("  London:   {}", check_eip1559_support::<spec::London>());

    // 演示编译时分支优化
    fn optimized_gas_calculation<S: Spec>(base_gas: u64) -> u64 {
        let mut total = base_gas;

        // 这些 if 语句在编译时就会被优化掉
        if S::ENABLE_ACCESS_LISTS {
            total += 100; // 访问列表额外成本
        }

        if S::ENABLE_EIP1559 {
            total += 50; // EIP-1559 额外成本
        }

        total
    }

    println!("\n⚡ 编译时优化演示 (基础 gas: 1000):");
    println!(
        "  Frontier 优化结果: {}",
        optimized_gas_calculation::<spec::Frontier>(1000)
    );
    println!(
        "  Berlin 优化结果:   {}",
        optimized_gas_calculation::<spec::Berlin>(1000)
    );
    println!(
        "  London 优化结果:   {}",
        optimized_gas_calculation::<spec::London>(1000)
    );
}

/// 演示 Gas 成本计算的实际差异
fn demonstrate_gas_differences() {
    println!("\n⛽ 演示 4: Gas 成本差异分析");
    println!("{}", "-".repeat(40));

    // 模拟复杂操作的 Gas 计算
    fn simulate_complex_operation<S: Spec>() -> u64 {
        let mut total_gas = 0;

        // 模拟一系列操作
        total_gas += S::GAS_CALL; // CALL 操作
        total_gas += S::GAS_SLOAD * 3; // 3 次存储读取
        total_gas += S::GAS_SSTORE_SET; // 1 次存储写入
        total_gas += S::GAS_CREATE; // CREATE 操作

        total_gas
    }

    println!("🧮 复杂操作的 Gas 成本计算:");
    println!("  操作序列: 1×CALL + 3×SLOAD + 1×SSTORE_SET + 1×CREATE");
    println!();

    let frontier_cost = simulate_complex_operation::<spec::Frontier>();
    let berlin_cost = simulate_complex_operation::<spec::Berlin>();
    let london_cost = simulate_complex_operation::<spec::London>();

    println!("  Frontier 总成本: {} gas", frontier_cost);
    println!(
        "  Berlin 总成本:   {} gas (+{})",
        berlin_cost,
        berlin_cost as i64 - frontier_cost as i64
    );
    println!(
        "  London 总成本:   {} gas (+{})",
        london_cost,
        london_cost as i64 - frontier_cost as i64
    );

    // 分析成本增长的原因
    println!("\n📈 成本增长分析:");
    println!("  Berlin 引入 EIP-2929，大幅提高冷访问成本");
    println!(
        "  CALL: {} -> {} (+{})",
        spec::Frontier::GAS_CALL,
        spec::Berlin::GAS_CALL,
        spec::Berlin::GAS_CALL as i64 - spec::Frontier::GAS_CALL as i64
    );
    println!(
        "  SLOAD: {} -> {} (+{})",
        spec::Frontier::GAS_SLOAD,
        spec::Berlin::GAS_SLOAD,
        spec::Berlin::GAS_SLOAD as i64 - spec::Frontier::GAS_SLOAD as i64
    );
}

/// 演示预编译合约支持
fn demonstrate_precompile_support() {
    println!("\n🔌 演示 5: 预编译合约支持");
    println!("{}", "-".repeat(40));

    fn analyze_precompiles<S: Spec>() {
        let precompiles = S::precompiles();
        println!("  {} 支持的预编译合约: {:?}", S::NAME, precompiles);
        println!("  数量: {} 个", precompiles.len());

        // 分析具体的预编译合约
        for &addr in precompiles {
            let name = match addr {
                1 => "ECDSA 恢复",
                2 => "SHA256 哈希",
                3 => "RIPEMD160 哈希",
                4 => "身份函数",
                5 => "模幂运算",
                6 => "椭圆曲线加法",
                7 => "椭圆曲线标量乘法",
                8 => "椭圆曲线配对",
                9 => "Blake2f 哈希",
                _ => "未知预编译",
            };
            println!("    地址 {}: {}", addr, name);
        }
        println!();
    }

    println!("📋 各规范的预编译合约支持:");
    analyze_precompiles::<spec::Frontier>();
    analyze_precompiles::<spec::Berlin>();
    analyze_precompiles::<spec::London>();

    // 演示预编译合约的使用
    fn is_precompile_available<S: Spec>(address: u8) -> bool {
        S::precompiles().contains(&address)
    }

    println!("🔍 特定预编译合约可用性检查:");
    let test_addresses = [1, 5, 9, 10];

    for addr in test_addresses {
        println!("  地址 {} 可用性:", addr);
        println!(
            "    Frontier: {}",
            if is_precompile_available::<spec::Frontier>(addr) {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "    Berlin:   {}",
            if is_precompile_available::<spec::Berlin>(addr) {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "    London:   {}",
            if is_precompile_available::<spec::London>(addr) {
                "✅"
            } else {
                "❌"
            }
        );
    }
}
