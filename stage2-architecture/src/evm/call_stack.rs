use crate::models::*;
use ethereum_types::{Address, U256};
use std::collections::HashMap;

/// EVM 调用帧
///
/// 每个调用帧代表一次函数调用的上下文，包含了该调用的所有必要信息。
/// 这是实现 EVM 调用栈的核心数据结构。
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// 调用者地址
    pub caller: Address,

    /// 被调用的代码地址
    pub code_address: Address,

    /// 接收 ETH 的地址（对于 DELEGATECALL 可能与 code_address 不同）
    pub to_address: Address,

    /// 调用传入的 ETH 数量
    pub value: U256,

    /// 调用数据
    pub data: Vec<u8>,

    /// Gas 限制
    pub gas_limit: u64,

    /// 已使用的 Gas
    pub gas_used: u64,

    /// 是否为只读调用（STATICCALL）
    pub read_only: bool,

    /// 调用类型
    pub call_type: CallType,

    /// 调用深度
    pub depth: usize,

    /// 返回数据偏移和大小
    pub return_data_offset: usize,
    pub return_data_size: usize,
}

/// 调用类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallType {
    /// CALL - 普通调用
    Call,
    /// CALLCODE - 使用调用者的存储
    CallCode,
    /// DELEGATECALL - 使用调用者的上下文
    DelegateCall,
    /// STATICCALL - 只读调用
    StaticCall,
    /// CREATE - 创建合约
    Create,
    /// CREATE2 - 确定性创建合约
    Create2,
}

impl CallFrame {
    /// 创建新的调用帧
    pub fn new_call(
        caller: Address,
        to: Address,
        value: U256,
        data: Vec<u8>,
        gas_limit: u64,
        call_type: CallType,
        depth: usize,
    ) -> Self {
        Self {
            caller,
            code_address: to,
            to_address: to,
            value,
            data,
            gas_limit,
            gas_used: 0,
            read_only: call_type == CallType::StaticCall,
            call_type,
            depth,
            return_data_offset: 0,
            return_data_size: 0,
        }
    }

    /// 创建 DELEGATECALL 帧
    pub fn new_delegate_call(
        caller: Address,
        code_address: Address,
        to_address: Address,
        value: U256,
        data: Vec<u8>,
        gas_limit: u64,
        depth: usize,
    ) -> Self {
        Self {
            caller,
            code_address,
            to_address,
            value,
            data,
            gas_limit,
            gas_used: 0,
            read_only: false,
            call_type: CallType::DelegateCall,
            depth,
            return_data_offset: 0,
            return_data_size: 0,
        }
    }

    /// 创建合约创建帧
    pub fn new_create(
        caller: Address,
        value: U256,
        init_code: Vec<u8>,
        gas_limit: u64,
        create_type: CallType,
        depth: usize,
    ) -> Self {
        Self {
            caller,
            code_address: Address::zero(), // 待计算
            to_address: Address::zero(),   // 待计算
            value,
            data: init_code,
            gas_limit,
            gas_used: 0,
            read_only: false,
            call_type: create_type,
            depth,
            return_data_offset: 0,
            return_data_size: 0,
        }
    }

    /// 消耗 Gas
    pub fn consume_gas(&mut self, gas: u64) -> Result<(), Error> {
        if self.gas_used + gas > self.gas_limit {
            return Err(Error::OutOfGas);
        }
        self.gas_used += gas;
        Ok(())
    }

    /// 获取剩余 Gas
    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    /// 检查是否可以修改状态
    pub fn can_modify_state(&self) -> bool {
        !self.read_only
    }
}

/// EVM 调用栈
///
/// 管理 EVM 执行过程中的调用层级，确保每个调用的上下文隔离和安全性。
#[derive(Debug)]
pub struct CallStack {
    /// 调用帧栈
    frames: Vec<CallFrame>,

    /// 当前调用深度
    current_depth: usize,

    /// 最大调用深度
    max_depth: usize,

    /// 调用历史（用于调试）
    call_history: Vec<String>,

    /// 是否记录调用历史
    record_history: bool,
}

impl CallStack {
    /// 创建新的调用栈
    pub fn new(max_depth: usize) -> Self {
        Self {
            frames: Vec::new(),
            current_depth: 0,
            max_depth,
            call_history: Vec::new(),
            record_history: false,
        }
    }

    /// 启用调用历史记录
    pub fn enable_history(&mut self) {
        self.record_history = true;
    }

    /// 获取调用历史
    pub fn get_history(&self) -> &[String] {
        &self.call_history
    }

    /// 推入新的调用帧
    pub fn push_frame(&mut self, mut frame: CallFrame) -> Result<(), Error> {
        // 检查调用深度限制
        if self.current_depth >= self.max_depth {
            return Err(Error::CallDepthExceeded);
        }

        // 设置正确的深度
        frame.depth = self.current_depth;

        // 记录调用历史
        if self.record_history {
            let history_entry = format!(
                "PUSH[{}] {:?} {} -> {} (gas: {})",
                self.current_depth,
                frame.call_type,
                format_address(frame.caller),
                format_address(frame.to_address),
                frame.gas_limit
            );
            self.call_history.push(history_entry);
        }

        // 推入帧并增加深度
        self.frames.push(frame);
        self.current_depth += 1;

        Ok(())
    }

    /// 弹出当前调用帧
    pub fn pop_frame(&mut self) -> Option<CallFrame> {
        if let Some(frame) = self.frames.pop() {
            self.current_depth = self.current_depth.saturating_sub(1);

            // 记录调用历史
            if self.record_history {
                let history_entry = format!(
                    "POP[{}] {:?} gas_used: {}",
                    frame.depth, frame.call_type, frame.gas_used
                );
                self.call_history.push(history_entry);
            }

            Some(frame)
        } else {
            None
        }
    }

    /// 获取当前调用帧的可变引用
    pub fn current_frame_mut(&mut self) -> Option<&mut CallFrame> {
        self.frames.last_mut()
    }

    /// 获取当前调用帧的引用
    pub fn current_frame(&self) -> Option<&CallFrame> {
        self.frames.last()
    }

    /// 获取调用者帧（上一层调用）
    pub fn caller_frame(&self) -> Option<&CallFrame> {
        if self.frames.len() >= 2 {
            self.frames.get(self.frames.len() - 2)
        } else {
            None
        }
    }

    /// 获取当前调用深度
    pub fn depth(&self) -> usize {
        self.current_depth
    }

    /// 检查栈是否为空
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// 获取栈大小
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// 回滚到指定深度（用于异常处理）
    pub fn rollback_to_depth(&mut self, target_depth: usize) -> Vec<CallFrame> {
        let mut rolled_back = Vec::new();

        while self.current_depth > target_depth && !self.frames.is_empty() {
            if let Some(frame) = self.pop_frame() {
                rolled_back.push(frame);
            }
        }

        rolled_back
    }

    /// 检查是否在只读上下文中
    pub fn is_in_static_context(&self) -> bool {
        self.frames.iter().any(|frame| frame.read_only)
    }

    /// 获取总的 Gas 使用量
    pub fn total_gas_used(&self) -> u64 {
        self.frames.iter().map(|frame| frame.gas_used).sum()
    }

    /// 格式化调用栈信息（用于调试）
    pub fn format_stack(&self) -> String {
        let mut result = String::new();
        result.push_str("=== Call Stack ===\n");

        for (i, frame) in self.frames.iter().enumerate() {
            result.push_str(&format!(
                "[{}] {:?} {} -> {} (gas: {}/{}, depth: {})\n",
                i,
                frame.call_type,
                format_address(frame.caller),
                format_address(frame.to_address),
                frame.gas_used,
                frame.gas_limit,
                frame.depth
            ));
        }

        if self.frames.is_empty() {
            result.push_str("(empty)\n");
        }

        result
    }
}

/// 调用栈管理器
///
/// 提供高级的调用栈操作，包括状态隔离、权限检查等。
#[derive(Debug)]
pub struct CallManager {
    /// 调用栈
    stack: CallStack,

    /// 返回数据缓存
    return_data: Vec<u8>,

    /// 状态变更记录（每个调用深度一个记录）
    state_changes: HashMap<usize, Vec<StateChange>>,

    /// 事件日志
    logs: Vec<Log>,
}

impl CallManager {
    /// 创建新的调用管理器
    pub fn new(max_depth: usize) -> Self {
        Self {
            stack: CallStack::new(max_depth),
            return_data: Vec::new(),
            state_changes: HashMap::new(),
            logs: Vec::new(),
        }
    }

    /// 开始新的调用
    pub fn begin_call(&mut self, frame: CallFrame) -> Result<(), Error> {
        let depth = frame.depth;

        // 推入调用帧
        self.stack.push_frame(frame)?;

        // 初始化该深度的状态变更记录
        self.state_changes.insert(depth, Vec::new());

        Ok(())
    }

    /// 结束当前调用
    pub fn end_call(&mut self, success: bool, return_data: Vec<u8>) -> Option<CallFrame> {
        if let Some(frame) = self.stack.pop_frame() {
            let depth = frame.depth;

            if success {
                // 调用成功，保留状态变更
                self.return_data = return_data;
            } else {
                // 调用失败，回滚状态变更
                self.rollback_state_changes(depth);
                self.return_data.clear();
            }

            // 清理该深度的状态变更记录
            self.state_changes.remove(&depth);

            Some(frame)
        } else {
            None
        }
    }

    /// 记录状态变更
    pub fn record_state_change(&mut self, change: StateChange) {
        if let Some(current_frame) = self.stack.current_frame() {
            let depth = current_frame.depth;
            self.state_changes.entry(depth).or_default().push(change);
        }
    }

    /// 回滚指定深度的状态变更
    fn rollback_state_changes(&mut self, depth: usize) {
        if let Some(changes) = self.state_changes.remove(&depth) {
            // 这里应该实际回滚状态变更
            // 简化实现，只是记录日志
            println!("回滚深度 {} 的 {} 个状态变更", depth, changes.len());
        }
    }

    /// 添加事件日志
    pub fn add_log(&mut self, log: Log) -> Result<(), Error> {
        // 检查是否在静态上下文中
        if self.stack.is_in_static_context() {
            return Err(Error::InvalidOpcode); // 静态调用不能产生日志
        }

        self.logs.push(log);
        Ok(())
    }

    /// 获取调用栈引用
    pub fn stack(&self) -> &CallStack {
        &self.stack
    }

    /// 获取可变调用栈引用
    pub fn stack_mut(&mut self) -> &mut CallStack {
        &mut self.stack
    }

    /// 获取返回数据
    pub fn return_data(&self) -> &[u8] {
        &self.return_data
    }

    /// 获取事件日志
    pub fn logs(&self) -> &[Log] {
        &self.logs
    }

    /// 检查权限
    pub fn check_permissions(&self, operation: &str) -> Result<(), Error> {
        if let Some(frame) = self.stack.current_frame() {
            match operation {
                "modify_state" if frame.read_only => {
                    return Err(Error::InvalidOpcode);
                }
                "emit_log" if frame.read_only => {
                    return Err(Error::InvalidOpcode);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 处理调用失败的清理工作
    pub fn handle_call_failure(&mut self, target_depth: usize) {
        // 回滚到目标深度
        let rolled_back = self.stack.rollback_to_depth(target_depth);

        // 清理回滚帧的状态变更
        for frame in rolled_back {
            self.rollback_state_changes(frame.depth);
        }

        // 清空返回数据
        self.return_data.clear();
    }
}

/// 辅助函数：格式化地址显示
fn format_address(addr: Address) -> String {
    if addr == Address::zero() {
        "0x0".to_string()
    } else {
        format!(
            "0x{:x}...{:x}",
            u32::from_be_bytes([addr[0], addr[1], addr[2], addr[3]]),
            u32::from_be_bytes([addr[16], addr[17], addr[18], addr[19]])
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_frame_creation() {
        let caller = Address::from([1u8; 20]);
        let to = Address::from([2u8; 20]);
        let value = U256::from(100);
        let data = vec![0x12, 0x34];

        let frame = CallFrame::new_call(caller, to, value, data.clone(), 10000, CallType::Call, 0);

        assert_eq!(frame.caller, caller);
        assert_eq!(frame.to_address, to);
        assert_eq!(frame.value, value);
        assert_eq!(frame.data, data);
        assert_eq!(frame.gas_limit, 10000);
        assert_eq!(frame.call_type, CallType::Call);
    }

    #[test]
    fn test_call_stack_operations() {
        let mut stack = CallStack::new(10);

        // 测试空栈
        assert!(stack.is_empty());
        assert_eq!(stack.depth(), 0);

        // 推入帧
        let frame1 = CallFrame::new_call(
            Address::from([1u8; 20]),
            Address::from([2u8; 20]),
            U256::zero(),
            vec![],
            10000,
            CallType::Call,
            0,
        );

        stack.push_frame(frame1).unwrap();
        assert_eq!(stack.depth(), 1);
        assert!(!stack.is_empty());

        // 弹出帧
        let popped = stack.pop_frame().unwrap();
        assert_eq!(popped.call_type, CallType::Call);
        assert!(stack.is_empty());
    }

    #[test]
    fn test_call_depth_limit() {
        let mut stack = CallStack::new(2);

        // 推入第一帧
        let frame1 = CallFrame::new_call(
            Address::from([1u8; 20]),
            Address::from([2u8; 20]),
            U256::zero(),
            vec![],
            10000,
            CallType::Call,
            0,
        );
        stack.push_frame(frame1).unwrap();

        // 推入第二帧
        let frame2 = CallFrame::new_call(
            Address::from([2u8; 20]),
            Address::from([3u8; 20]),
            U256::zero(),
            vec![],
            10000,
            CallType::Call,
            1,
        );
        stack.push_frame(frame2).unwrap();

        // 尝试推入第三帧应该失败
        let frame3 = CallFrame::new_call(
            Address::from([3u8; 20]),
            Address::from([4u8; 20]),
            U256::zero(),
            vec![],
            10000,
            CallType::Call,
            2,
        );

        assert!(matches!(
            stack.push_frame(frame3),
            Err(Error::CallDepthExceeded)
        ));
    }
}
