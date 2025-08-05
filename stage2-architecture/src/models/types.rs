use ethereum_types::{Address, H256, U256};

/// 基础账户信息
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountInfo {
    pub balance: U256,
    pub nonce: u64,
    pub code_hash: H256,
    pub code: Option<Vec<u8>>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::zero(),
            nonce: 0,
            code_hash: H256::zero(),
            code: None,
        }
    }
}

/// 字节码表示
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bytecode {
    pub bytes: Vec<u8>,
    pub hash: H256,
}

impl Bytecode {
    pub fn new(bytes: Vec<u8>) -> Self {
        let hash = keccak_hash::keccak(&bytes);
        Self { bytes, hash }
    }
}

/// 创建方案
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateScheme {
    /// CREATE 操作码
    Legacy { caller: Address },
    /// CREATE2 操作码
    Create2 {
        caller: Address,
        code_hash: H256,
        salt: H256,
    },
    /// 固定地址（如预编译合约）
    Fixed(Address),
}

/// 调用方案
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallScheme {
    /// CALL 操作码
    Call,
    /// CALLCODE 操作码
    CallCode,
    /// DELEGATECALL 操作码
    DelegateCall,
    /// STATICCALL 操作码
    StaticCall,
}

/// 交易信息
#[derive(Debug, Clone)]
pub struct Transaction {
    pub caller: Address,
    pub to: Option<Address>,
    pub value: U256,
    pub data: Vec<u8>,
    pub gas_limit: u64,
    pub gas_price: U256,
}

/// 执行环境
#[derive(Debug, Clone)]
pub struct Environment {
    pub block_number: U256,
    pub block_timestamp: U256,
    pub block_difficulty: U256,
    pub block_gas_limit: u64,
    pub chain_id: U256,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            block_number: U256::from(1),
            block_timestamp: U256::from(1000000),
            block_difficulty: U256::from(1000),
            block_gas_limit: 30_000_000,
            chain_id: U256::from(1),
        }
    }
}

/// 执行结果
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub gas_used: u64,
    pub return_data: Vec<u8>,
    pub logs: Vec<Log>,
}

/// 日志条目
#[derive(Debug, Clone)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
    pub data: Vec<u8>,
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

/// EVM 错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    OutOfGas,
    StackUnderflow,
    StackOverflow,
    InvalidOpcode,
    InvalidJump,
    CallDepthExceeded,
    CreateCollision,
    OutOfMemory,
    DatabaseError,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::OutOfGas => write!(f, "Out of gas"),
            Error::StackUnderflow => write!(f, "Stack underflow"),
            Error::StackOverflow => write!(f, "Stack overflow"),
            Error::InvalidOpcode => write!(f, "Invalid opcode"),
            Error::InvalidJump => write!(f, "Invalid jump"),
            Error::CallDepthExceeded => write!(f, "Call depth exceeded"),
            Error::CreateCollision => write!(f, "Create collision"),
            Error::OutOfMemory => write!(f, "Out of memory"),
            Error::DatabaseError => write!(f, "Database error"),
        }
    }
}

impl std::error::Error for Error {}
