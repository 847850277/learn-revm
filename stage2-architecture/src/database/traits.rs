use crate::models::*;
use ethereum_types::{Address, H256, U256};

/// 数据库 trait - 定义 EVM 与存储层的交互接口
///
/// 这个 trait 抽象了 EVM 需要的所有数据库操作，
/// 使得 EVM 可以与不同的存储后端配合工作。
pub trait Database {
    type Error: std::fmt::Debug;

    /// 获取账户基础信息（余额、nonce、代码哈希）
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// 根据代码哈希获取字节码
    fn code_by_hash(&mut self, code_hash: H256) -> Result<Bytecode, Self::Error>;

    /// 读取账户存储槽的值
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error>;

    /// 获取账户代码（包括代码哈希计算）
    fn code(&mut self, address: Address) -> Result<Bytecode, Self::Error> {
        let basic = self.basic(address)?;
        match basic {
            Some(acc) if acc.code_hash != H256::zero() => self.code_by_hash(acc.code_hash),
            _ => Ok(Bytecode::new(vec![])),
        }
    }

    /// 检查账户是否存在
    fn exists(&mut self, address: Address) -> Result<bool, Self::Error> {
        Ok(self.basic(address)?.is_some())
    }
}

/// 可变数据库 trait - 支持状态修改操作
pub trait DatabaseCommit: Database {
    /// 提交状态变更
    fn commit(&mut self, changes: Vec<StateChange>) -> Result<(), Self::Error>;
}

/// 状态变更类型
#[derive(Debug, Clone)]
pub enum StateChange {
    /// 创建新账户
    CreateAccount { address: Address, info: AccountInfo },
    /// 删除账户
    DeleteAccount { address: Address },
    /// 更新账户余额
    UpdateBalance { address: Address, balance: U256 },
    /// 更新账户 nonce
    UpdateNonce { address: Address, nonce: u64 },
    /// 设置账户代码
    SetCode { address: Address, code: Bytecode },
    /// 更新存储槽
    UpdateStorage {
        address: Address,
        index: U256,
        value: U256,
    },
}

/// 数据库事务支持
pub trait DatabaseTransaction: Database {
    type Transaction;

    /// 开始事务
    fn begin_transaction(&mut self) -> Self::Transaction;

    /// 提交事务
    fn commit_transaction(&mut self, tx: Self::Transaction) -> Result<(), Self::Error>;

    /// 回滚事务
    fn rollback_transaction(&mut self, tx: Self::Transaction) -> Result<(), Self::Error>;
}
