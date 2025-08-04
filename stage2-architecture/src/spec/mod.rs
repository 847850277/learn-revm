/// EVM 规范 trait
///
/// 这个 trait 定义了不同以太坊硬分叉的规范参数，
/// 所有参数都是编译时常量，确保零运行时成本。
pub trait Spec: Clone + 'static {
    /// 规范名称
    const NAME: &'static str;

    // === Gas 成本常量 ===

    /// CALL 系列指令的基础 gas 成本
    const GAS_CALL: u64;

    /// SLOAD 指令的 gas 成本
    const GAS_SLOAD: u64;

    /// SSTORE 设置新值的 gas 成本
    const GAS_SSTORE_SET: u64;

    /// SSTORE 重置值的 gas 成本
    const GAS_SSTORE_RESET: u64;

    /// SSTORE 清除值的 gas 退款
    const GAS_SSTORE_CLEAR_REFUND: i64;

    /// CREATE 指令的基础 gas 成本
    const GAS_CREATE: u64;

    /// 每字节代码部署的 gas 成本
    const GAS_CODE_DEPOSIT: u64;

    // === EIP 特性开关 ===

    /// 是否启用 CREATE2 指令 (EIP-1014)
    const ENABLE_CREATE2: bool;

    /// 是否启用 CHAINID 指令 (EIP-1344)
    const ENABLE_CHAINID: bool;

    /// 是否启用 SELFBALANCE 指令 (EIP-1884)
    const ENABLE_SELFBALANCE: bool;

    /// 是否启用访问列表 (EIP-2930)
    const ENABLE_ACCESS_LISTS: bool;

    /// 是否启用 EIP-1559 手续费机制
    const ENABLE_EIP1559: bool;

    // === 系统限制参数 ===

    /// 栈最大深度
    const STACK_LIMIT: usize;

    /// 内存最大大小
    const MEMORY_LIMIT: usize;

    /// 调用栈最大深度
    const CALL_DEPTH_LIMIT: usize;

    /// 代码最大大小
    const MAX_CODE_SIZE: usize;

    // === 预编译合约支持 ===

    /// 获取支持的预编译合约地址列表
    fn precompiles() -> &'static [u8];
}

/// Berlin 硬分叉规范 (2021年4月)
///
/// 主要特性：
/// - EIP-2929: Gas 成本增加，访问列表
/// - EIP-2930: 可选访问列表交易类型
/// - EIP-2718: 类型化交易包络
#[derive(Clone, Debug)]
pub struct Berlin;

impl Spec for Berlin {
    const NAME: &'static str = "Berlin";

    // Berlin 的 Gas 成本（受 EIP-2929 影响）
    const GAS_CALL: u64 = 700; // 冷访问成本更高
    const GAS_SLOAD: u64 = 800; // 冷存储读取成本
    const GAS_SSTORE_SET: u64 = 20000;
    const GAS_SSTORE_RESET: u64 = 5000;
    const GAS_SSTORE_CLEAR_REFUND: i64 = 4800;
    const GAS_CREATE: u64 = 32000;
    const GAS_CODE_DEPOSIT: u64 = 200;

    // Berlin 支持的 EIP 特性
    const ENABLE_CREATE2: bool = true;
    const ENABLE_CHAINID: bool = true;
    const ENABLE_SELFBALANCE: bool = true;
    const ENABLE_ACCESS_LISTS: bool = true; // EIP-2930
    const ENABLE_EIP1559: bool = false; // London 才有

    // 系统限制
    const STACK_LIMIT: usize = 1024;
    const MEMORY_LIMIT: usize = 0x1FFFFFFE0;
    const CALL_DEPTH_LIMIT: usize = 1024;
    const MAX_CODE_SIZE: usize = 0x6000; // EIP-170

    fn precompiles() -> &'static [u8] {
        // Berlin 支持 1-9 号预编译合约
        &[1, 2, 3, 4, 5, 6, 7, 8, 9]
    }
}

/// London 硬分叉规范 (2021年8月)
///
/// 主要特性：
/// - EIP-1559: 新的手续费机制，引入 base fee
/// - EIP-3198: BASEFEE 操作码
/// - EIP-3529: 减少 SSTORE 清除的退款
#[derive(Clone, Debug)]
pub struct London;

impl Spec for London {
    const NAME: &'static str = "London";

    // London 继承 Berlin 的 Gas 成本，部分调整
    const GAS_CALL: u64 = 700;
    const GAS_SLOAD: u64 = 800;
    const GAS_SSTORE_SET: u64 = 20000;
    const GAS_SSTORE_RESET: u64 = 5000;
    const GAS_SSTORE_CLEAR_REFUND: i64 = 0; // EIP-3529: 取消清除退款
    const GAS_CREATE: u64 = 32000;
    const GAS_CODE_DEPOSIT: u64 = 200;

    // London 的 EIP 特性
    const ENABLE_CREATE2: bool = true;
    const ENABLE_CHAINID: bool = true;
    const ENABLE_SELFBALANCE: bool = true;
    const ENABLE_ACCESS_LISTS: bool = true;
    const ENABLE_EIP1559: bool = true; // 新增 EIP-1559

    // 系统限制与 Berlin 相同
    const STACK_LIMIT: usize = 1024;
    const MEMORY_LIMIT: usize = 0x1FFFFFFE0;
    const CALL_DEPTH_LIMIT: usize = 1024;
    const MAX_CODE_SIZE: usize = 0x6000;

    fn precompiles() -> &'static [u8] {
        // London 支持 1-9 号预编译合约
        &[1, 2, 3, 4, 5, 6, 7, 8, 9]
    }
}

/// 旧版规范（用于对比）
#[derive(Clone, Debug)]
pub struct Frontier;

impl Spec for Frontier {
    const NAME: &'static str = "Frontier";

    // Frontier 的原始 Gas 成本
    const GAS_CALL: u64 = 40; // 原始低成本
    const GAS_SLOAD: u64 = 200; // 原始成本
    const GAS_SSTORE_SET: u64 = 20000;
    const GAS_SSTORE_RESET: u64 = 5000;
    const GAS_SSTORE_CLEAR_REFUND: i64 = 15000; // 高退款
    const GAS_CREATE: u64 = 32000;
    const GAS_CODE_DEPOSIT: u64 = 200;

    // Frontier 不支持现代 EIP 特性
    const ENABLE_CREATE2: bool = false;
    const ENABLE_CHAINID: bool = false;
    const ENABLE_SELFBALANCE: bool = false;
    const ENABLE_ACCESS_LISTS: bool = false;
    const ENABLE_EIP1559: bool = false;

    // 系统限制
    const STACK_LIMIT: usize = 1024;
    const MEMORY_LIMIT: usize = 0x1FFFFFFE0;
    const CALL_DEPTH_LIMIT: usize = 1024;
    const MAX_CODE_SIZE: usize = usize::MAX; // 无限制

    fn precompiles() -> &'static [u8] {
        // Frontier 仅支持 1-4 号预编译合约
        &[1, 2, 3, 4]
    }
}

/// 规范比较工具
pub struct SpecComparison;

impl SpecComparison {
    /// 比较两个规范的 Gas 成本差异
    pub fn compare_gas_costs<S1: Spec, S2: Spec>() -> Vec<(String, u64, u64, i64)> {
        vec![
            (
                "CALL".to_string(),
                S1::GAS_CALL,
                S2::GAS_CALL,
                S2::GAS_CALL as i64 - S1::GAS_CALL as i64,
            ),
            (
                "SLOAD".to_string(),
                S1::GAS_SLOAD,
                S2::GAS_SLOAD,
                S2::GAS_SLOAD as i64 - S1::GAS_SLOAD as i64,
            ),
            (
                "SSTORE_SET".to_string(),
                S1::GAS_SSTORE_SET,
                S2::GAS_SSTORE_SET,
                S2::GAS_SSTORE_SET as i64 - S1::GAS_SSTORE_SET as i64,
            ),
        ]
    }

    /// 比较两个规范的特性支持
    pub fn compare_features<S1: Spec, S2: Spec>() -> Vec<(String, bool, bool)> {
        vec![
            (
                "CREATE2".to_string(),
                S1::ENABLE_CREATE2,
                S2::ENABLE_CREATE2,
            ),
            (
                "CHAINID".to_string(),
                S1::ENABLE_CHAINID,
                S2::ENABLE_CHAINID,
            ),
            (
                "SELFBALANCE".to_string(),
                S1::ENABLE_SELFBALANCE,
                S2::ENABLE_SELFBALANCE,
            ),
            (
                "ACCESS_LISTS".to_string(),
                S1::ENABLE_ACCESS_LISTS,
                S2::ENABLE_ACCESS_LISTS,
            ),
            (
                "EIP1559".to_string(),
                S1::ENABLE_EIP1559,
                S2::ENABLE_EIP1559,
            ),
        ]
    }
}
