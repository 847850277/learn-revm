use ethereum_types::{Address, H256, U256};
use stage2_architecture::*;

/// 练习 3: EVM 调用栈系统深入实践
///
/// 本练习将展示：
/// 1. EVM 调用栈的内部结构和工作原理
/// 2. 不同类型调用的处理机制
/// 3. 调用深度限制和安全性保证
/// 4. 状态隔离和权限管理
/// 5. 调用失败的回滚机制

fn main() {
    println!("🎯 练习 3: EVM 调用栈系统深入实践");
    println!("{}", "=".repeat(60));

    // 演示 1: 基础调用栈操作
    demonstrate_basic_call_stack();

    // 演示 2: 不同类型的调用
    demonstrate_call_types();

    // 演示 3: 调用深度限制和安全性
    demonstrate_call_depth_limits();

    // 演示 4: 状态隔离和权限管理
    demonstrate_state_isolation();

    // 演示 5: 调用失败和回滚机制
    demonstrate_failure_rollback();

    // 演示 6: 复杂调用场景
    demonstrate_complex_call_scenarios();

    println!("\n🎉 练习 3 完成！您已经深入理解了 EVM 调用栈的核心机制。");
}

/// 演示基础调用栈操作
fn demonstrate_basic_call_stack() {
    println!("\n📚 演示 1: 基础调用栈操作");
    println!("{}", "-".repeat(50));

    let mut call_stack = CallStack::new(10);
    call_stack.enable_history();

    println!("🔧 创建调用栈 (最大深度: 10)");
    println!("   初始状态: {}", call_stack.format_stack().trim());

    // 创建几个测试地址
    let user = Address::from([1u8; 20]);
    let contract_a = Address::from([2u8; 20]);
    let contract_b = Address::from([3u8; 20]);

    // 第一层调用：用户调用合约A
    println!("\n📞 推入调用 1: 用户 -> 合约A");
    let frame1 = CallFrame::new_call(
        user,
        contract_a,
        U256::from(100),
        vec![0x12, 0x34],
        50000,
        CallType::Call,
        0,
    );

    call_stack.push_frame(frame1).unwrap();
    println!("   当前深度: {}", call_stack.depth());
    println!("   栈大小: {}", call_stack.len());

    // 第二层调用：合约A调用合约B
    println!("\n📞 推入调用 2: 合约A -> 合约B");
    let frame2 = CallFrame::new_call(
        contract_a,
        contract_b,
        U256::from(50),
        vec![0x56, 0x78],
        30000,
        CallType::Call,
        1,
    );

    call_stack.push_frame(frame2).unwrap();
    println!("   当前深度: {}", call_stack.depth());
    println!("   当前调用栈:");
    println!("{}", call_stack.format_stack());

    // 获取当前调用帧信息
    if let Some(current) = call_stack.current_frame() {
        println!("📋 当前调用帧详情:");
        println!("   调用者: {}", format_address_short(current.caller));
        println!("   被调用者: {}", format_address_short(current.to_address));
        println!("   调用类型: {:?}", current.call_type);
        println!("   Gas 限制: {}", current.gas_limit);
        println!("   剩余 Gas: {}", current.remaining_gas());
    }

    // 弹出调用
    println!("\n🔄 弹出调用 2");
    if let Some(popped) = call_stack.pop_frame() {
        println!(
            "   弹出的调用: {:?} (深度: {})",
            popped.call_type, popped.depth
        );
    }

    println!("\n🔄 弹出调用 1");
    if let Some(popped) = call_stack.pop_frame() {
        println!(
            "   弹出的调用: {:?} (深度: {})",
            popped.call_type, popped.depth
        );
    }

    println!("\n📝 调用历史记录:");
    for (i, history) in call_stack.get_history().iter().enumerate() {
        println!("   {}: {}", i + 1, history);
    }
}

/// 演示不同类型的调用
fn demonstrate_call_types() {
    println!("\n🔄 演示 2: 不同类型的调用");
    println!("{}", "-".repeat(50));

    let mut call_stack = CallStack::new(10);
    call_stack.enable_history();

    let caller = Address::from([1u8; 20]);
    let target = Address::from([2u8; 20]);
    let value = U256::from(100);
    let data = vec![0x12, 0x34, 0x56, 0x78];

    // 1. CALL - 普通调用
    println!("📞 1. CALL 调用");
    let call_frame = CallFrame::new_call(
        caller,
        target,
        value,
        data.clone(),
        50000,
        CallType::Call,
        0,
    );
    call_stack.push_frame(call_frame).unwrap();

    if let Some(frame) = call_stack.current_frame() {
        println!("   特点: 转移 ETH, 使用目标合约的存储");
        println!("   调用者: {}", format_address_short(frame.caller));
        println!("   目标地址: {}", format_address_short(frame.to_address));
        println!("   传输价值: {} ETH", frame.value);
        println!("   只读模式: {}", frame.read_only);
    }
    call_stack.pop_frame();

    // 2. STATICCALL - 只读调用
    println!("\n📞 2. STATICCALL 调用");
    let static_frame = CallFrame::new_call(
        caller,
        target,
        U256::zero(),
        data.clone(),
        30000,
        CallType::StaticCall,
        0,
    );
    call_stack.push_frame(static_frame).unwrap();

    if let Some(frame) = call_stack.current_frame() {
        println!("   特点: 不能修改状态, 不能转移 ETH");
        println!("   只读模式: {}", frame.read_only);
        println!("   传输价值: {} ETH (必须为0)", frame.value);
        println!(
            "   权限检查: {}",
            if frame.can_modify_state() {
                "可修改状态"
            } else {
                "禁止修改状态"
            }
        );
    }
    call_stack.pop_frame();

    // 3. DELEGATECALL - 委托调用
    println!("\n📞 3. DELEGATECALL 调用");
    let code_address = Address::from([3u8; 20]);
    let delegate_frame =
        CallFrame::new_delegate_call(caller, code_address, target, value, data.clone(), 40000, 0);
    call_stack.push_frame(delegate_frame).unwrap();

    if let Some(frame) = call_stack.current_frame() {
        println!("   特点: 使用调用者的存储和余额");
        println!("   调用者: {}", format_address_short(frame.caller));
        println!("   代码地址: {}", format_address_short(frame.code_address));
        println!("   存储地址: {}", format_address_short(frame.to_address));
        println!("   上下文保持: 调用者的身份和存储");
    }
    call_stack.pop_frame();

    // 4. CREATE - 合约创建
    println!("\n📞 4. CREATE 调用");
    let init_code = vec![0x60, 0x80, 0x60, 0x40, 0x52]; // 简单的初始化代码
    let create_frame = CallFrame::new_create(
        caller,
        value,
        init_code.clone(),
        100000,
        CallType::Create,
        0,
    );
    call_stack.push_frame(create_frame).unwrap();

    if let Some(frame) = call_stack.current_frame() {
        println!("   特点: 创建新合约");
        println!("   创建者: {}", format_address_short(frame.caller));
        println!("   初始化代码长度: {} 字节", frame.data.len());
        println!("   创建价值: {} ETH", frame.value);
    }
    call_stack.pop_frame();

    // 5. CREATE2 - 确定性创建
    println!("\n📞 5. CREATE2 调用");
    let create2_frame =
        CallFrame::new_create(caller, value, init_code, 120000, CallType::Create2, 0);
    call_stack.push_frame(create2_frame).unwrap();

    if let Some(frame) = call_stack.current_frame() {
        println!("   特点: 确定性地址创建");
        println!("   创建者: {}", format_address_short(frame.caller));
        println!("   地址可预测: 基于 salt 和代码哈希");
    }
    call_stack.pop_frame();

    println!("\n📊 调用类型总结:");
    println!("   • CALL: 普通调用，可转移 ETH，使用目标存储");
    println!("   • STATICCALL: 只读调用，不能修改状态");
    println!("   • DELEGATECALL: 委托调用，使用调用者上下文");
    println!("   • CREATE: 合约创建，生成新地址");
    println!("   • CREATE2: 确定性创建，地址可预测");
}

/// 演示调用深度限制和安全性
fn demonstrate_call_depth_limits() {
    println!("\n🛡️ 演示 3: 调用深度限制和安全性");
    println!("{}", "-".repeat(50));

    // 创建限制深度为3的调用栈
    let mut call_stack = CallStack::new(3);
    call_stack.enable_history();

    println!("🔧 创建限制深度为 3 的调用栈");

    let addresses = [
        Address::from([1u8; 20]),
        Address::from([2u8; 20]),
        Address::from([3u8; 20]),
        Address::from([4u8; 20]),
    ];

    // 尝试推入超过限制的调用
    for i in 0..4 {
        println!("\n📞 尝试推入调用 {} (深度: {})", i + 1, i);

        let frame = CallFrame::new_call(
            addresses[i],
            addresses[(i + 1) % 4],
            U256::from(100),
            vec![],
            10000,
            CallType::Call,
            i,
        );

        match call_stack.push_frame(frame) {
            Ok(()) => {
                println!("   ✅ 成功推入调用 (当前深度: {})", call_stack.depth());
            }
            Err(Error::CallDepthExceeded) => {
                println!("   ❌ 调用深度超限！拒绝执行");
                println!("   🛡️ 安全机制生效: 防止无限递归攻击");
                break;
            }
            Err(e) => {
                println!("   ❌ 其他错误: {:?}", e);
                break;
            }
        }
    }

    println!("\n📊 当前调用栈状态:");
    println!("{}", call_stack.format_stack());

    // 演示回滚操作
    println!("\n🔄 演示回滚操作:");
    println!("   回滚前深度: {}", call_stack.depth());

    let rolled_back = call_stack.rollback_to_depth(1);
    println!("   回滚到深度 1");
    println!("   回滚后深度: {}", call_stack.depth());
    println!("   回滚的调用数: {}", rolled_back.len());

    for (i, frame) in rolled_back.iter().enumerate() {
        println!(
            "     回滚调用 {}: {:?} (深度: {})",
            i + 1,
            frame.call_type,
            frame.depth
        );
    }

    println!("\n💡 深度限制的重要性:");
    println!("   • 防止无限递归导致的栈溢出");
    println!("   • 限制 Gas 消耗，避免 DoS 攻击");
    println!("   • 保护网络节点的计算资源");
    println!("   • 确保 EVM 执行的可预测性");
}

/// 演示状态隔离和权限管理
fn demonstrate_state_isolation() {
    println!("\n🔒 演示 4: 状态隔离和权限管理");
    println!("{}", "-".repeat(50));

    let mut call_manager = CallManager::new(10);

    let user = Address::from([1u8; 20]);
    let contract = Address::from([2u8; 20]);

    // 普通调用 - 可以修改状态
    println!("📞 1. 普通 CALL 调用");
    let call_frame = CallFrame::new_call(
        user,
        contract,
        U256::from(100),
        vec![],
        50000,
        CallType::Call,
        0,
    );
    call_manager.begin_call(call_frame).unwrap();

    // 检查权限
    match call_manager.check_permissions("modify_state") {
        Ok(()) => println!("   ✅ 可以修改状态"),
        Err(e) => println!("   ❌ 无法修改状态: {:?}", e),
    }

    match call_manager.check_permissions("emit_log") {
        Ok(()) => println!("   ✅ 可以发出事件日志"),
        Err(e) => println!("   ❌ 无法发出事件日志: {:?}", e),
    }

    // 记录状态变更
    let state_change = StateChange::UpdateBalance {
        address: contract,
        balance: U256::from(200),
    };
    call_manager.record_state_change(state_change);
    println!("   📝 记录状态变更: 更新余额");

    // 添加事件日志
    let log = Log {
        address: contract,
        topics: vec![H256::from([1u8; 32])],
        data: vec![0x12, 0x34],
    };

    match call_manager.add_log(log) {
        Ok(()) => println!("   📋 成功添加事件日志"),
        Err(e) => println!("   ❌ 添加事件日志失败: {:?}", e),
    }

    // 结束普通调用
    call_manager.end_call(true, vec![0x42]);

    // 静态调用 - 不能修改状态
    println!("\n📞 2. STATICCALL 静态调用");
    let static_frame = CallFrame::new_call(
        user,
        contract,
        U256::zero(),
        vec![],
        30000,
        CallType::StaticCall,
        0,
    );
    call_manager.begin_call(static_frame).unwrap();

    // 检查权限
    match call_manager.check_permissions("modify_state") {
        Ok(()) => println!("   ✅ 可以修改状态"),
        Err(e) => println!("   ❌ 无法修改状态: {:?}", e),
    }

    match call_manager.check_permissions("emit_log") {
        Ok(()) => println!("   ✅ 可以发出事件日志"),
        Err(e) => println!("   ❌ 无法发出事件日志: {:?}", e),
    }

    // 尝试添加事件日志（应该失败）
    let static_log = Log {
        address: contract,
        topics: vec![H256::from([2u8; 32])],
        data: vec![0x56, 0x78],
    };

    match call_manager.add_log(static_log) {
        Ok(()) => println!("   📋 成功添加事件日志"),
        Err(e) => println!("   ❌ 添加事件日志失败: {:?}", e),
    }

    call_manager.end_call(true, vec![0x84]);

    println!("\n📊 权限管理总结:");
    println!("   • 普通调用: 可以修改状态、发出日志");
    println!("   • 静态调用: 只读访问，禁止状态修改");
    println!("   • 委托调用: 使用调用者的权限");
    println!("   • 权限检查在运行时执行");

    println!("\n📋 最终日志数量: {}", call_manager.logs().len());
}

/// 演示调用失败和回滚机制
fn demonstrate_failure_rollback() {
    println!("\n↩️ 演示 5: 调用失败和回滚机制");
    println!("{}", "-".repeat(50));

    let mut call_manager = CallManager::new(10);
    call_manager.stack_mut().enable_history();

    let user = Address::from([1u8; 20]);
    let contract_a = Address::from([2u8; 20]);
    let contract_b = Address::from([3u8; 20]);

    println!("🎬 场景: 嵌套调用中的失败处理");

    // 第一层调用：用户 -> 合约A
    println!("\n📞 开始调用 1: 用户 -> 合约A");
    let frame1 = CallFrame::new_call(
        user,
        contract_a,
        U256::from(100),
        vec![],
        50000,
        CallType::Call,
        0,
    );
    call_manager.begin_call(frame1).unwrap();

    // 记录第一层的状态变更
    let change1 = StateChange::UpdateBalance {
        address: contract_a,
        balance: U256::from(150),
    };
    call_manager.record_state_change(change1);
    println!("   📝 记录状态变更 1: 合约A余额更新");

    // 第二层调用：合约A -> 合约B
    println!("\n📞 开始调用 2: 合约A -> 合约B");
    let frame2 = CallFrame::new_call(
        contract_a,
        contract_b,
        U256::from(50),
        vec![],
        30000,
        CallType::Call,
        1,
    );
    call_manager.begin_call(frame2).unwrap();

    // 记录第二层的状态变更
    let change2 = StateChange::UpdateBalance {
        address: contract_b,
        balance: U256::from(200),
    };
    call_manager.record_state_change(change2);
    println!("   📝 记录状态变更 2: 合约B余额更新");

    // 第三层调用：合约B -> 合约A (可能导致失败)
    println!("\n📞 开始调用 3: 合约B -> 合约A");
    let frame3 = CallFrame::new_call(
        contract_b,
        contract_a,
        U256::from(25),
        vec![],
        15000,
        CallType::Call,
        2,
    );
    call_manager.begin_call(frame3).unwrap();

    // 记录第三层的状态变更
    let change3 = StateChange::UpdateStorage {
        address: contract_a,
        index: U256::from(0),
        value: U256::from(42),
    };
    call_manager.record_state_change(change3);
    println!("   📝 记录状态变更 3: 合约A存储更新");

    println!("\n📊 调用前状态:");
    println!("{}", call_manager.stack().format_stack());

    // 模拟第三层调用失败
    println!("\n💥 第三层调用失败！");
    println!("   原因: Gas 不足 / 执行异常");

    // 结束失败的调用
    let failed_frame = call_manager.end_call(false, vec![]).unwrap();
    println!(
        "   🔄 回滚调用 3: {:?} (深度: {})",
        failed_frame.call_type, failed_frame.depth
    );
    println!("   📝 状态变更 3 已回滚");

    // 第二层调用也可能因为子调用失败而失败
    println!("\n⚠️ 第二层调用决定也失败（受子调用影响）");
    let failed_frame2 = call_manager.end_call(false, vec![]).unwrap();
    println!(
        "   🔄 回滚调用 2: {:?} (深度: {})",
        failed_frame2.call_type, failed_frame2.depth
    );
    println!("   📝 状态变更 2 已回滚");

    // 第一层调用成功完成
    println!("\n✅ 第一层调用成功完成");
    let success_frame = call_manager.end_call(true, vec![0x01]).unwrap();
    println!(
        "   🎯 完成调用 1: {:?} (深度: {})",
        success_frame.call_type, success_frame.depth
    );
    println!("   📝 状态变更 1 已保留");

    println!("\n📊 最终状态:");
    println!("   调用栈深度: {}", call_manager.stack().depth());
    println!("   返回数据: {:?}", hex::encode(call_manager.return_data()));

    println!("\n📝 调用历史:");
    for (i, history) in call_manager.stack().get_history().iter().enumerate() {
        println!("   {}: {}", i + 1, history);
    }

    println!("\n💡 回滚机制要点:");
    println!("   • 失败的调用会回滚其所有状态变更");
    println!("   • 父调用可以选择是否受子调用失败影响");
    println!("   • 成功的调用保留其状态变更");
    println!("   • 回滚是原子性的，确保一致性");
}

/// 演示复杂调用场景
fn demonstrate_complex_call_scenarios() {
    println!("\n🎭 演示 6: 复杂调用场景");
    println!("{}", "-".repeat(50));

    let mut call_manager = CallManager::new(5);
    call_manager.stack_mut().enable_history();

    println!("🎬 场景: 混合调用类型的复杂交互");

    let user = Address::from([1u8; 20]);
    let proxy = Address::from([2u8; 20]);
    let implementation = Address::from([3u8; 20]);
    let library = Address::from([4u8; 20]);

    // 1. 用户调用代理合约
    println!("\n📞 1. 用户 -> 代理合约 (CALL)");
    let frame1 = CallFrame::new_call(
        user,
        proxy,
        U256::from(100),
        vec![0x12, 0x34],
        100000,
        CallType::Call,
        0,
    );
    call_manager.begin_call(frame1).unwrap();

    // 2. 代理合约委托调用实现合约
    println!("\n📞 2. 代理合约 -> 实现合约 (DELEGATECALL)");
    let frame2 = CallFrame::new_delegate_call(
        proxy,
        implementation,
        proxy,
        U256::from(100),
        vec![0x56, 0x78],
        80000,
        1,
    );
    call_manager.begin_call(frame2).unwrap();

    println!("   💡 DELEGATECALL 特点:");
    if let Some(frame) = call_manager.stack().current_frame() {
        println!(
            "     - 调用者保持为: {}",
            format_address_short(frame.caller)
        );
        println!(
            "     - 代码来自: {}",
            format_address_short(frame.code_address)
        );
        println!(
            "     - 存储地址: {}",
            format_address_short(frame.to_address)
        );
        println!("     - 使用代理合约的存储空间");
    }

    // 3. 实现合约静态调用库合约
    println!("\n📞 3. 实现合约 -> 库合约 (STATICCALL)");
    let frame3 = CallFrame::new_call(
        implementation,
        library,
        U256::zero(),
        vec![0x9a, 0xbc],
        20000,
        CallType::StaticCall,
        2,
    );
    call_manager.begin_call(frame3).unwrap();

    println!("   💡 STATICCALL 特点:");
    if let Some(frame) = call_manager.stack().current_frame() {
        println!("     - 只读调用: {}", frame.read_only);
        println!("     - 不能修改状态");
        println!("     - 不能转移 ETH");
    }

    // 4. 库合约尝试创建新合约 (应该失败)
    println!("\n📞 4. 库合约尝试 CREATE (应该在静态上下文中失败)");
    println!("   ❌ 静态调用上下文禁止创建操作");
    println!("   🛡️ 权限检查阻止了潜在的状态修改");

    // 静态调用成功返回
    println!("\n✅ 静态调用成功返回");
    call_manager.end_call(true, vec![0xde, 0xad, 0xbe, 0xef]);

    // DELEGATECALL 成功返回
    println!("\n✅ DELEGATECALL 成功返回");
    call_manager.end_call(true, vec![0xca, 0xfe, 0xba, 0xbe]);

    // 主调用成功返回
    println!("\n✅ 主调用成功返回");
    call_manager.end_call(true, vec![0x42, 0x42]);

    println!("\n📊 复杂调用链总结:");
    println!("   调用链: 用户 -> 代理 -> 实现 -> 库");
    println!("   调用类型: CALL -> DELEGATECALL -> STATICCALL");
    println!("   最大深度: 3");
    println!("   最终返回: {:?}", hex::encode(call_manager.return_data()));

    println!("\n📝 完整调用历史:");
    for (i, history) in call_manager.stack().get_history().iter().enumerate() {
        println!("   {}: {}", i + 1, history);
    }

    println!("\n🎯 复杂场景的关键点:");
    println!("   • 代理模式: DELEGATECALL 保持调用者上下文");
    println!("   • 库调用: STATICCALL 确保纯函数特性");
    println!("   • 权限继承: 子调用受父调用权限限制");
    println!("   • 状态隔离: 每层调用都有独立的权限和状态");
}

/// 辅助函数：简化地址显示
fn format_address_short(addr: Address) -> String {
    if addr == Address::zero() {
        "0x0".to_string()
    } else {
        format!(
            "0x{:02x}{:02x}...{:02x}{:02x}",
            addr[0], addr[1], addr[18], addr[19]
        )
    }
}
