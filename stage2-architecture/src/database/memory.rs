use crate::database::traits::*;
use crate::models::*;
use ethereum_types::{Address, H256, U256};
use std::collections::HashMap;

/// 内存数据库实现
///
/// 这是一个简单的内存数据库，将所有状态存储在内存中。
/// 主要用于测试和演示，不适合生产环境。
#[derive(Debug, Clone)]
pub struct InMemoryDB {
    /// 账户信息存储
    accounts: HashMap<Address, AccountInfo>,

    /// 存储槽数据 (address, slot) -> value
    storage: HashMap<(Address, U256), U256>,

    /// 代码存储 code_hash -> bytecode
    code: HashMap<H256, Bytecode>,

    /// 是否记录访问日志
    log_access: bool,

    /// 访问日志
    access_log: Vec<String>,
}

impl InMemoryDB {
    /// 创建新的内存数据库
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            storage: HashMap::new(),
            code: HashMap::new(),
            log_access: false,
            access_log: Vec::new(),
        }
    }

    /// 启用访问日志记录
    pub fn enable_logging(&mut self) {
        self.log_access = true;
        self.access_log.clear();
    }

    /// 获取访问日志
    pub fn get_access_log(&self) -> &[String] {
        &self.access_log
    }

    /// 预设账户信息（用于测试）
    pub fn insert_account(&mut self, address: Address, info: AccountInfo) {
        if let Some(ref code) = info.code {
            let bytecode = Bytecode::new(code.clone());
            self.code.insert(bytecode.hash, bytecode);
        }
        self.accounts.insert(address, info);
    }

    /// 预设存储值（用于测试）
    pub fn insert_storage(&mut self, address: Address, index: U256, value: U256) {
        self.storage.insert((address, index), value);
    }

    /// 获取所有账户（用于调试）
    pub fn get_all_accounts(&self) -> &HashMap<Address, AccountInfo> {
        &self.accounts
    }

    /// 获取账户存储（用于调试）
    pub fn get_account_storage(&self, address: Address) -> Vec<(U256, U256)> {
        self.storage
            .iter()
            .filter(|((addr, _), _)| *addr == address)
            .map(|((_, slot), value)| (*slot, *value))
            .collect()
    }

    /// 记录访问日志
    fn log(&mut self, operation: &str) {
        if self.log_access {
            self.access_log.push(operation.to_string());
        }
    }
}

impl Default for InMemoryDB {
    fn default() -> Self {
        Self::new()
    }
}

impl Database for InMemoryDB {
    type Error = ();

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.log(&format!("BASIC: {:#x}", address));
        Ok(self.accounts.get(&address).cloned())
    }

    fn code_by_hash(&mut self, code_hash: H256) -> Result<Bytecode, Self::Error> {
        self.log(&format!("CODE_BY_HASH: {:#x}", code_hash));

        if code_hash == H256::zero() {
            return Ok(Bytecode::new(vec![]));
        }

        Ok(self
            .code
            .get(&code_hash)
            .cloned()
            .unwrap_or_else(|| Bytecode::new(vec![])))
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.log(&format!("STORAGE: {:#x}[{:#x}]", address, index));
        Ok(self
            .storage
            .get(&(address, index))
            .copied()
            .unwrap_or(U256::zero()))
    }
}

impl DatabaseCommit for InMemoryDB {
    fn commit(&mut self, changes: Vec<StateChange>) -> Result<(), Self::Error> {
        self.log(&format!("COMMIT: {} changes", changes.len()));

        for change in changes {
            match change {
                StateChange::CreateAccount { address, info } => {
                    if let Some(ref code) = info.code {
                        let bytecode = Bytecode::new(code.clone());
                        self.code.insert(bytecode.hash, bytecode);
                    }
                    self.accounts.insert(address, info);
                }
                StateChange::DeleteAccount { address } => {
                    self.accounts.remove(&address);
                    // 清理相关存储
                    self.storage.retain(|(addr, _), _| *addr != address);
                }
                StateChange::UpdateBalance { address, balance } => {
                    if let Some(account) = self.accounts.get_mut(&address) {
                        account.balance = balance;
                    }
                }
                StateChange::UpdateNonce { address, nonce } => {
                    if let Some(account) = self.accounts.get_mut(&address) {
                        account.nonce = nonce;
                    }
                }
                StateChange::SetCode { address, code } => {
                    self.code.insert(code.hash, code.clone());
                    if let Some(account) = self.accounts.get_mut(&address) {
                        account.code_hash = code.hash;
                        account.code = Some(code.bytes);
                    }
                }
                StateChange::UpdateStorage {
                    address,
                    index,
                    value,
                } => {
                    if value == U256::zero() {
                        self.storage.remove(&(address, index));
                    } else {
                        self.storage.insert((address, index), value);
                    }
                }
            }
        }
        Ok(())
    }
}

/// 测试辅助函数
impl InMemoryDB {
    /// 创建预填充的测试数据库
    pub fn with_test_data() -> Self {
        let mut db = Self::new();

        // 添加一些测试账户
        let addr1 = Address::from([1u8; 20]);
        let addr2 = Address::from([2u8; 20]);

        // 账户1: 普通账户，有余额
        db.insert_account(
            addr1,
            AccountInfo {
                balance: U256::from(1000u64),
                nonce: 5,
                code_hash: H256::zero(),
                code: None,
            },
        );

        // 账户2: 合约账户，有代码
        let contract_code = vec![0x60, 0x80, 0x60, 0x40, 0x52]; // 简单的合约代码
        let code_hash = keccak_hash::keccak(&contract_code);
        db.insert_account(
            addr2,
            AccountInfo {
                balance: U256::from(500u64),
                nonce: 1,
                code_hash,
                code: Some(contract_code.clone()),
            },
        );

        // 为合约账户添加一些存储
        db.insert_storage(addr2, U256::from(0), U256::from(42));
        db.insert_storage(addr2, U256::from(1), U256::from(100));

        db
    }
}
