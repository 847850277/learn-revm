use ethereum_types::{Address, H256, U256};
use stage2_architecture::*;

/// 练习 2: 模块化 EVM 引擎实践
///
/// 本练习将展示：
/// 1. 泛型 EVM 引擎的使用方法
/// 2. 不同规范的 EVM 实例创建
/// 3. 数据库后端的可插拔性
/// 4. 交易执行的完整流程
/// 5. 规范参数对执行结果的影响

fn main() {
    println!("🎯 练习 2: 模块化 EVM 引擎实践");
    println!("{}", "=".repeat(60));

    // 演示 1: 创建不同规范的 EVM 实例
    demonstrate_spec_based_evm_creation();

    // 演示 2: 数据库后端的可插拔性
    demonstrate_database_pluggability();

    // 演示 3: 交易执行流程
    demonstrate_transaction_execution();

    // 演示 4: 规范差异对执行的影响
    demonstrate_spec_impact_on_execution();

    // 演示 5: EVM 引擎的扩展性
    demonstrate_evm_extensibility();

    println!("\n🎉 练习 2 完成！您已经掌握了模块化 EVM 引擎的核心概念。");
}

/// 演示基于规范的 EVM 实例创建
fn demonstrate_spec_based_evm_creation() {
    println!("\n📦 演示 1: 基于规范的 EVM 实例创建");
    println!("{}", "-".repeat(50));

    // 创建测试数据库
    let db = InMemoryDB::with_test_data();

    // 创建不同规范的 EVM 实例
    println!("🏗️ 创建不同规范的 EVM 实例:");

    // Frontier EVM
    let mut frontier_evm = create_frontier_evm(db.clone());
    println!("  ✅ Frontier EVM 创建成功");
    frontier_evm.check_feature_support();

    // Berlin EVM
    let mut berlin_evm = create_berlin_evm(db.clone());
    println!("  ✅ Berlin EVM 创建成功");
    berlin_evm.check_feature_support();

    // London EVM
    let mut london_evm = create_london_evm(db.clone());
    println!("  ✅ London EVM 创建成功");
    london_evm.check_feature_support();

    println!("\n💡 关键观察:");
    println!("  • 相同的数据库可以与不同规范的 EVM 配合");
    println!("  • 每个 EVM 实例都有独立的规范参数");
    println!("  • 类型系统确保规范不会混用");
}

/// 演示数据库后端的可插拔性
fn demonstrate_database_pluggability() {
    println!("\n🔌 演示 2: 数据库后端的可插拔性");
    println!("{}", "-".repeat(50));

    // 创建不同配置的数据库
    println!("📊 创建不同类型的数据库:");

    // 空数据库
    let empty_db = InMemoryDB::new();
    println!("  📦 空数据库创建成功");

    // 预填充数据库
    let test_db = InMemoryDB::with_test_data();
    println!("  📦 测试数据库创建成功");

    // 带日志的数据库
    let mut logging_db = InMemoryDB::new();
    logging_db.enable_logging();
    println!("  📦 日志数据库创建成功");

    // 将不同数据库与同一规范的 EVM 配合
    println!("\n🔗 数据库与 EVM 的组合:");

    let berlin_evm_empty = create_berlin_evm(empty_db);
    println!("  ✅ Berlin EVM + 空数据库");

    let berlin_evm_test = create_berlin_evm(test_db);
    println!("  ✅ Berlin EVM + 测试数据库");

    let berlin_evm_logging = create_berlin_evm(logging_db);
    println!("  ✅ Berlin EVM + 日志数据库");

    // 展示数据库内容
    println!("\n📋 测试数据库内容预览:");
    let db_with_data = InMemoryDB::with_test_data();
    let accounts = db_with_data.get_all_accounts();
    for (addr, account) in accounts {
        println!("  账户 {:#x}:", addr);
        println!("    余额: {}", account.balance);
        println!("    Nonce: {}", account.nonce);
        println!("    有代码: {}", account.code.is_some());
    }
}

/// 演示完整的交易执行流程
fn demonstrate_transaction_execution() {
    println!("\n🚀 演示 3: 交易执行流程");
    println!("{}", "-".repeat(50));

    // 准备测试环境
    let mut db = InMemoryDB::with_test_data();
    db.enable_logging();
    let mut evm = create_berlin_evm(db);

    // 准备测试账户
    let caller = Address::from([1u8; 20]);
    let contract = Address::from([2u8; 20]);

    println!("📋 执行环境准备:");
    println!("  调用者: {:#x}", caller);
    println!("  合约:   {:#x}", contract);

    // 执行调用交易
    println!("\n📞 执行 CALL 交易:");
    let call_tx = Transaction {
        caller,
        to: Some(contract),
        value: U256::from(100),
        data: vec![0x12, 0x34, 0x56, 0x78],
        gas_limit: 100000,
        gas_price: U256::from(20_000_000_000u64), // 20 gwei
    };

    let call_result = evm.transact(call_tx).unwrap();
    println!("📊 调用结果:");
    println!("  成功: {}", call_result.success);
    println!("  Gas 使用: {}", call_result.gas_used);
    println!("  返回数据: {:?}", hex::encode(&call_result.return_data));

    // 执行创建交易
    println!("\n🏭 执行 CREATE 交易:");
    let create_tx = Transaction {
        caller,
        to: None,
        value: U256::from(0),
        data: vec![0x60, 0x80, 0x60, 0x40, 0x52, 0x00], // 简单的合约字节码
        gas_limit: 200000,
        gas_price: U256::from(20_000_000_000u64),
    };

    let create_result = evm.transact(create_tx).unwrap();
    println!("📊 创建结果:");
    println!("  成功: {}", create_result.success);
    println!("  Gas 使用: {}", create_result.gas_used);
    println!("  新合约地址: {}", hex::encode(&create_result.return_data));

    // 显示数据库访问日志
    println!("\n📝 数据库访问日志:");
    let access_log = evm.database().get_access_log();
    for (i, log_entry) in access_log.iter().enumerate() {
        println!("  {}: {}", i + 1, log_entry);
    }
}

/// 演示规范差异对执行的影响
fn demonstrate_spec_impact_on_execution() {
    println!("\n⚖️ 演示 4: 规范差异对执行的影响");
    println!("{}", "-".repeat(50));

    // 创建相同的交易
    let caller = Address::from([1u8; 20]);
    let contract = Address::from([2u8; 20]);

    let tx = Transaction {
        caller,
        to: Some(contract),
        value: U256::from(100),
        data: vec![0x12, 0x34],
        gas_limit: 100000,
        gas_price: U256::from(20_000_000_000u64),
    };

    println!("📊 相同交易在不同规范下的执行结果:");

    // Frontier 执行
    let mut frontier_evm = create_frontier_evm(InMemoryDB::with_test_data());
    let frontier_result = frontier_evm.transact(tx.clone()).unwrap();

    // Berlin 执行
    let mut berlin_evm = create_berlin_evm(InMemoryDB::with_test_data());
    let berlin_result = berlin_evm.transact(tx.clone()).unwrap();

    // London 执行
    let mut london_evm = create_london_evm(InMemoryDB::with_test_data());
    let london_result = london_evm.transact(tx.clone()).unwrap();

    println!("\n📈 Gas 消耗对比:");
    println!("  Frontier: {} gas", frontier_result.gas_used);
    println!(
        "  Berlin:   {} gas (+{})",
        berlin_result.gas_used,
        berlin_result.gas_used as i64 - frontier_result.gas_used as i64
    );
    println!(
        "  London:   {} gas (+{})",
        london_result.gas_used,
        london_result.gas_used as i64 - frontier_result.gas_used as i64
    );

    println!("\n📊 执行结果分析:");
    println!(
        "  所有规范都成功执行: {}",
        frontier_result.success && berlin_result.success && london_result.success
    );

    let gas_increase_berlin = berlin_result.gas_used as f64 / frontier_result.gas_used as f64 - 1.0;
    println!(
        "  Berlin vs Frontier Gas 增长: {:.1}%",
        gas_increase_berlin * 100.0
    );

    println!("\n💡 差异原因:");
    println!("  • EIP-2929 提高了冷账户/存储访问成本");
    println!(
        "  • CALL 指令成本从 {} 增加到 {}",
        spec::Frontier::GAS_CALL,
        spec::Berlin::GAS_CALL
    );
}

/// 演示 EVM 引擎的扩展性
fn demonstrate_evm_extensibility() {
    println!("\n🔧 演示 5: EVM 引擎的扩展性");
    println!("{}", "-".repeat(50));

    // 自定义数据库实现示例
    println!("🛠️ 扩展性展示:");

    // 1. 可以轻松替换数据库实现
    println!("  ✅ 可插拔数据库后端");
    println!("     - InMemoryDB: 内存存储，适合测试");
    println!("     - 可扩展: 磁盘存储、网络存储等");

    // 2. 可以添加新的规范
    println!("  ✅ 可扩展规范系统");
    println!("     - 现有: Frontier, Berlin, London");
    println!("     - 可添加: Shanghai, Cancun, Prague 等");

    // 3. 类型安全的组合
    println!("  ✅ 类型安全的组合");
    println!("     - 编译时确保规范与功能匹配");
    println!("     - 防止运行时错误");

    // 演示泛型的威力
    fn generic_evm_operation<SPEC: Spec, DB: Database>(
        evm: &mut EVM<SPEC, DB>,
        operation_name: &str,
    ) {
        println!("    执行 {} (规范: {})", operation_name, SPEC::NAME);
        // 这里可以添加通用的 EVM 操作逻辑
    }

    println!("\n🎯 泛型操作演示:");
    let mut berlin_evm = create_berlin_evm(InMemoryDB::new());
    let mut london_evm = create_london_evm(InMemoryDB::new());

    generic_evm_operation(&mut berlin_evm, "余额查询");
    generic_evm_operation(&mut london_evm, "余额查询");

    println!("\n🚀 模块化架构优势总结:");
    println!("  🔹 关注点分离: 规范、存储、执行逻辑独立");
    println!("  🔹 可测试性: 每个组件都可以独立测试");
    println!("  🔹 可扩展性: 新规范和后端可以无缝集成");
    println!("  🔹 类型安全: 编译时确保正确性");
    println!("  🔹 性能优化: 编译时特化，零运行时成本");
}
