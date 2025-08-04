use crate::database::Database;
use crate::models::*;
use crate::spec::Spec;
use ethereum_types::{Address, U256};
use std::marker::PhantomData;

/// EVM 执行机器状态
#[derive(Debug, Clone)]
pub struct Machine {
    /// 程序计数器
    pub pc: usize,

    /// 执行栈
    pub stack: Vec<U256>,

    /// 内存
    pub memory: Vec<u8>,

    /// 返回数据
    pub return_data: Vec<u8>,

    /// 剩余 Gas
    pub gas: u64,
}

impl Machine {
    pub fn new(gas: u64) -> Self {
        Self {
            pc: 0,
            stack: Vec::new(),
            memory: Vec::new(),
            return_data: Vec::new(),
            gas,
        }
    }

    /// 栈操作：推入值
    pub fn push(&mut self, value: U256) -> Result<(), Error> {
        if self.stack.len() >= 1024 {
            return Err(Error::StackOverflow);
        }
        self.stack.push(value);
        Ok(())
    }

    /// 栈操作：弹出值
    pub fn pop(&mut self) -> Result<U256, Error> {
        self.stack.pop().ok_or(Error::StackUnderflow)
    }

    /// 内存操作：扩展内存
    pub fn expand_memory(&mut self, offset: usize, size: usize) -> Result<(), Error> {
        let required_size = offset + size;
        if required_size > self.memory.len() {
            // 内存按 32 字节对齐扩展
            let aligned_size = (required_size + 31) / 32 * 32;
            self.memory.resize(aligned_size, 0);
        }
        Ok(())
    }

    /// 内存操作：读取内存
    pub fn memory_read(&self, offset: usize, size: usize) -> Result<Vec<u8>, Error> {
        if offset + size > self.memory.len() {
            return Err(Error::OutOfMemory);
        }
        Ok(self.memory[offset..offset + size].to_vec())
    }

    /// 内存操作：写入内存
    pub fn memory_write(&mut self, offset: usize, data: &[u8]) -> Result<(), Error> {
        self.expand_memory(offset, data.len())?;
        self.memory[offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }

    /// 消耗 Gas
    pub fn use_gas(&mut self, gas: u64) -> Result<(), Error> {
        if self.gas < gas {
            return Err(Error::OutOfGas);
        }
        self.gas -= gas;
        Ok(())
    }
}

/// 模块化 EVM 引擎
///
/// 这个 EVM 引擎展示了模块化设计的核心理念：
/// - 使用泛型参数分离关注点
/// - SPEC 参数提供编译时优化
/// - DB 参数支持可插拔存储
pub struct EVM<SPEC: Spec, DB: Database> {
    /// 数据库后端
    database: DB,

    /// 执行环境
    env: Environment,

    /// 执行机器状态
    machine: Machine,

    /// 规范类型标记（零大小类型）
    _spec: PhantomData<SPEC>,
}

impl<SPEC: Spec, DB: Database> EVM<SPEC, DB> {
    /// 创建新的 EVM 实例
    pub fn new(database: DB, env: Environment) -> Self {
        Self {
            database,
            env,
            machine: Machine::new(0), // gas 将在执行时设置
            _spec: PhantomData,
        }
    }

    /// 执行交易
    pub fn transact(&mut self, tx: Transaction) -> Result<ExecutionResult, Error> {
        // 设置初始 gas
        self.machine.gas = tx.gas_limit;

        println!("🚀 开始执行交易 (规范: {})", SPEC::NAME);
        println!("   调用者: {:#x}", tx.caller);
        println!("   Gas 限制: {}", tx.gas_limit);

        // 检查栈限制（使用规范参数）
        if self.machine.stack.len() > SPEC::STACK_LIMIT {
            return Err(Error::StackOverflow);
        }

        // 根据交易类型执行
        let result = match tx.to {
            Some(to) => {
                println!("   类型: CALL to {:#x}", to);
                self.execute_call(tx.caller, to, tx.value, &tx.data)
            }
            None => {
                println!("   类型: CREATE");
                self.execute_create(tx.caller, tx.value, &tx.data)
            }
        };

        match result {
            Ok(return_data) => {
                let gas_used = tx.gas_limit - self.machine.gas;
                println!("✅ 交易执行成功，Gas 使用: {}", gas_used);

                Ok(ExecutionResult {
                    success: true,
                    gas_used,
                    return_data,
                    logs: Vec::new(),
                })
            }
            Err(e) => {
                let gas_used = tx.gas_limit - self.machine.gas;
                println!("❌ 交易执行失败: {}, Gas 使用: {}", e, gas_used);

                Ok(ExecutionResult {
                    success: false,
                    gas_used,
                    return_data: Vec::new(),
                    logs: Vec::new(),
                })
            }
        }
    }

    /// 执行调用
    fn execute_call(
        &mut self,
        caller: Address,
        to: Address,
        value: U256,
        data: &[u8],
    ) -> Result<Vec<u8>, Error> {
        // 消耗 CALL 的基础 gas（使用规范参数）
        self.machine.use_gas(SPEC::GAS_CALL)?;

        println!("   CALL gas 成本: {}", SPEC::GAS_CALL);

        // 检查目标账户
        let account = self.database.basic(to).map_err(|_| Error::DatabaseError)?;

        match account {
            Some(acc) if acc.code_hash != Default::default() => {
                println!("   调用合约 {:#x}", to);

                // 获取合约代码
                let code = self.database.code(to).map_err(|_| Error::DatabaseError)?;

                println!("   合约代码长度: {} 字节", code.bytes.len());

                // 模拟简单的合约执行
                if !code.bytes.is_empty() {
                    // 这里可以添加真正的字节码解释器
                    // 现在只是返回一些模拟数据
                    Ok(vec![0x42, 0x00]) // 模拟返回值
                } else {
                    Ok(Vec::new())
                }
            }
            _ => {
                println!("   调用外部账户 {:#x}", to);
                // 外部账户调用，没有代码执行
                Ok(Vec::new())
            }
        }
    }

    /// 执行创建
    fn execute_create(
        &mut self,
        caller: Address,
        value: U256,
        init_code: &[u8],
    ) -> Result<Vec<u8>, Error> {
        // 消耗 CREATE 的基础 gas（使用规范参数）
        self.machine.use_gas(SPEC::GAS_CREATE)?;

        println!("   CREATE gas 成本: {}", SPEC::GAS_CREATE);

        // 检查代码大小限制
        if init_code.len() > SPEC::MAX_CODE_SIZE {
            return Err(Error::OutOfMemory);
        }

        // 计算新合约地址
        let contract_address = self.calculate_create_address(caller, 1); // 简化的 nonce

        println!("   新合约地址: {:#x}", contract_address);
        println!("   初始化代码长度: {} 字节", init_code.len());

        // 计算代码部署成本
        let deploy_cost = (init_code.len() as u64) * SPEC::GAS_CODE_DEPOSIT;
        self.machine.use_gas(deploy_cost)?;

        println!("   代码部署 gas 成本: {}", deploy_cost);

        // 模拟合约创建成功
        Ok(contract_address.as_bytes().to_vec())
    }

    /// 计算 CREATE 地址
    fn calculate_create_address(&self, caller: Address, nonce: u64) -> Address {
        // 简化实现：使用 caller + nonce 计算地址
        // 实际实现应该使用 RLP 编码 + Keccak256
        let mut addr_bytes = [0u8; 20];
        let caller_bytes = caller.as_bytes();
        let nonce_bytes = nonce.to_be_bytes();

        for i in 0..20 {
            addr_bytes[i] = caller_bytes[i] ^ nonce_bytes[i % 8];
        }

        Address::from(addr_bytes)
    }

    /// 获取数据库引用（用于测试）
    pub fn database(&self) -> &DB {
        &self.database
    }

    /// 获取可变数据库引用（用于测试）
    pub fn database_mut(&mut self) -> &mut DB {
        &mut self.database
    }

    /// 获取当前机器状态（用于调试）
    pub fn machine(&self) -> &Machine {
        &self.machine
    }

    /// 检查规范特性支持
    pub fn check_feature_support(&self) {
        println!("🔧 {} 规范特性支持:", SPEC::NAME);
        println!(
            "   CREATE2: {}",
            if SPEC::ENABLE_CREATE2 { "✅" } else { "❌" }
        );
        println!(
            "   CHAINID: {}",
            if SPEC::ENABLE_CHAINID { "✅" } else { "❌" }
        );
        println!(
            "   SELFBALANCE: {}",
            if SPEC::ENABLE_SELFBALANCE {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "   ACCESS_LISTS: {}",
            if SPEC::ENABLE_ACCESS_LISTS {
                "✅"
            } else {
                "❌"
            }
        );
        println!(
            "   EIP1559: {}",
            if SPEC::ENABLE_EIP1559 { "✅" } else { "❌" }
        );

        println!("📊 {} 规范限制:", SPEC::NAME);
        println!("   栈限制: {}", SPEC::STACK_LIMIT);
        println!("   内存限制: {:#x}", SPEC::MEMORY_LIMIT);
        println!("   调用深度限制: {}", SPEC::CALL_DEPTH_LIMIT);
        println!("   代码大小限制: {}", SPEC::MAX_CODE_SIZE);
    }
}

/// 演示模块化设计的工厂函数
pub fn create_berlin_evm<DB: Database>(database: DB) -> EVM<crate::spec::Berlin, DB> {
    use crate::spec::Berlin;
    EVM::<Berlin, DB>::new(database, Environment::default())
}

pub fn create_london_evm<DB: Database>(database: DB) -> EVM<crate::spec::London, DB> {
    use crate::spec::London;
    EVM::<London, DB>::new(database, Environment::default())
}

pub fn create_frontier_evm<DB: Database>(database: DB) -> EVM<crate::spec::Frontier, DB> {
    use crate::spec::Frontier;
    EVM::<Frontier, DB>::new(database, Environment::default())
}
