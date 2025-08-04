// 🎮 EVM 完整指令序列模拟练习 - 理解完整程序执行流程
// 🔄 学习如何将多个指令组合成完整的 EVM 程序

use std::collections::HashMap;

// 简化的栈实现
#[derive(Debug, Clone)]
struct SimpleStack {
    data: Vec<u64>,
}

impl SimpleStack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, value: u64) -> Result<(), &'static str> {
        if self.data.len() >= 1024 {
            return Err("Stack overflow");
        }
        self.data.push(value);
        Ok(())
    }

    fn pop(&mut self) -> Result<u64, &'static str> {
        match self.data.pop() {
            Some(value) => Ok(value),
            None => Err("Stack underflow"),
        }
    }

    fn peek(&self) -> Option<u64> {
        self.data.last().copied()
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// 完整的指令集
#[derive(Debug, Clone)]
enum Instruction {
    // 栈操作
    Push(u64),

    // 算术指令
    Add,
    Sub,
    Mul,
    Div,

    // 比较指令
    Lt, // 小于
    Gt, // 大于
    Eq, // 等于

    // 逻辑指令
    And,
    Or,
    Not,

    // 内存指令
    MStore,
    MLoad,

    // 跳转指令
    Jump,
    JumpI,
    JumpDest,

    // 控制指令
    Stop,
}

// 内存实现
#[derive(Debug, Clone)]
struct SimpleMemory {
    data: HashMap<u64, u64>,
    size: u64,
}

impl SimpleMemory {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            size: 0,
        }
    }

    fn store(&mut self, offset: u64, value: u64) -> Result<(), &'static str> {
        let required_size = offset + 32;
        if required_size > self.size {
            self.size = ((required_size + 31) / 32) * 32; // 32字节对齐
        }
        self.data.insert(offset, value);
        Ok(())
    }

    fn load(&self, offset: u64) -> u64 {
        self.data.get(&offset).copied().unwrap_or(0)
    }

    fn current_size(&self) -> u64 {
        self.size
    }
}

// 跳转验证器
#[derive(Debug)]
struct JumpValidator {
    valid_destinations: Vec<bool>,
}

impl JumpValidator {
    fn new(instructions: &[Instruction]) -> Self {
        let mut valid_destinations = vec![false; instructions.len()];

        for (i, instruction) in instructions.iter().enumerate() {
            if matches!(instruction, Instruction::JumpDest) {
                valid_destinations[i] = true;
            }
        }

        Self { valid_destinations }
    }

    fn is_valid_destination(&self, pc: usize) -> bool {
        self.valid_destinations.get(pc).copied().unwrap_or(false)
    }
}

// 完整的 EVM 模拟器
#[derive(Debug)]
struct CompleteEVM {
    stack: SimpleStack,
    memory: SimpleMemory,
    instructions: Vec<Instruction>,
    validator: JumpValidator,
    pc: usize,
    gas_used: u64,
    gas_limit: u64,
    step_count: usize,
}

impl CompleteEVM {
    fn new(instructions: Vec<Instruction>, gas_limit: u64) -> Self {
        let validator = JumpValidator::new(&instructions);
        Self {
            stack: SimpleStack::new(),
            memory: SimpleMemory::new(),
            validator,
            instructions,
            pc: 0,
            gas_used: 0,
            gas_limit,
            step_count: 0,
        }
    }

    fn consume_gas(&mut self, gas: u64) -> Result<(), &'static str> {
        if self.gas_used + gas > self.gas_limit {
            return Err("Out of gas");
        }
        self.gas_used += gas;
        Ok(())
    }

    fn step(&mut self) -> Result<bool, &'static str> {
        if self.pc >= self.instructions.len() {
            return Ok(false);
        }

        self.step_count += 1;
        let instruction = self.instructions[self.pc].clone();

        println!(
            "\n🔧 步骤 {} [PC={}]: {:?}",
            self.step_count, self.pc, instruction
        );

        match instruction {
            Instruction::Push(value) => {
                self.consume_gas(3)?;
                self.stack.push(value)?;
                println!("  📥 PUSH: 将 {} 推入栈", value);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Add => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a + b;
                self.stack.push(result)?;
                println!("  ➕ ADD: {} + {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Sub => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.saturating_sub(b);
                self.stack.push(result)?;
                println!("  ➖ SUB: {} - {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Mul => {
                self.consume_gas(5)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a * b;
                self.stack.push(result)?;
                println!("  ✖️ MUL: {} * {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +5", self.stack.data);
                self.pc += 1;
            }

            Instruction::Div => {
                self.consume_gas(5)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = if b == 0 { 0 } else { a / b };
                self.stack.push(result)?;
                println!("  ➗ DIV: {} / {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +5", self.stack.data);
                self.pc += 1;
            }

            Instruction::Lt => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = if a < b { 1 } else { 0 };
                self.stack.push(result)?;
                println!("  🔍 LT: {} < {} = {} ({})", a, b, result, result == 1);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Gt => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = if a > b { 1 } else { 0 };
                self.stack.push(result)?;
                println!("  🔍 GT: {} > {} = {} ({})", a, b, result, result == 1);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Eq => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = if a == b { 1 } else { 0 };
                self.stack.push(result)?;
                println!("  🔍 EQ: {} == {} = {} ({})", a, b, result, result == 1);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::And => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a & b;
                self.stack.push(result)?;
                println!("  🔗 AND: {} & {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Or => {
                self.consume_gas(3)?;
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a | b;
                self.stack.push(result)?;
                println!("  🔗 OR: {} | {} = {}", a, b, result);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Not => {
                self.consume_gas(3)?;
                let a = self.stack.pop()?;
                let result = if a == 0 { 1 } else { 0 };
                self.stack.push(result)?;
                println!("  🚫 NOT: !{} = {} (逻辑非)", a, result);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::MStore => {
                self.consume_gas(3)?;
                let offset = self.stack.pop()?;
                let value = self.stack.pop()?;
                self.memory.store(offset, value)?;
                println!("  💾 MSTORE: 在偏移 {} 存储值 {}", offset, value);
                println!(
                    "     内存大小: {} 字节 | Gas: +3",
                    self.memory.current_size()
                );
                self.pc += 1;
            }

            Instruction::MLoad => {
                self.consume_gas(3)?;
                let offset = self.stack.pop()?;
                let value = self.memory.load(offset);
                self.stack.push(value)?;
                println!("  💾 MLOAD: 从偏移 {} 加载值 {}", offset, value);
                println!("     栈状态: {:?} | Gas: +3", self.stack.data);
                self.pc += 1;
            }

            Instruction::Jump => {
                self.consume_gas(8)?;
                let dest = self.stack.pop()? as usize;
                if !self.validator.is_valid_destination(dest) {
                    return Err("Invalid jump destination");
                }
                println!("  🚀 JUMP: 跳转到 PC = {}", dest);
                println!("     验证通过，执行跳转 | Gas: +8");
                self.pc = dest;
            }

            Instruction::JumpI => {
                self.consume_gas(10)?;
                let dest = self.stack.pop()? as usize;
                let condition = self.stack.pop()?;

                if condition != 0 {
                    if !self.validator.is_valid_destination(dest) {
                        return Err("Invalid jump destination");
                    }
                    println!("  🤔 JUMPI: 条件 {} 为真，跳转到 PC = {}", condition, dest);
                    self.pc = dest;
                } else {
                    println!("  🤔 JUMPI: 条件 {} 为假，继续顺序执行", condition);
                    self.pc += 1;
                }
                println!("     Gas: +10");
            }

            Instruction::JumpDest => {
                self.consume_gas(1)?;
                println!("  🏁 JUMPDEST: 有效跳转目标标记");
                println!("     这是一个跳转目标点 | Gas: +1");
                self.pc += 1;
            }

            Instruction::Stop => {
                println!("  🛑 STOP: 程序停止执行");
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn run(&mut self) -> Result<(), &'static str> {
        println!("🚀 开始执行完整 EVM 程序");
        println!("   指令总数: {}", self.instructions.len());
        println!("   Gas 限制: {}", self.gas_limit);

        while self.step()? {
            // 每10步打印一次状态摘要
            if self.step_count % 10 == 0 {
                self.print_state_summary();
            }
        }

        Ok(())
    }

    fn print_state_summary(&self) {
        println!("\n📊 执行状态摘要 (步骤 {}):", self.step_count);
        println!("   当前 PC: {}", self.pc);
        println!("   栈深度: {}", self.stack.len());
        println!("   已使用 Gas: {} / {}", self.gas_used, self.gas_limit);
        println!("   内存大小: {} 字节", self.memory.current_size());
    }

    fn print_final_state(&self) {
        println!("\n🎯 程序执行完成！");
        println!("===========================================");
        println!("   总执行步数: {}", self.step_count);
        println!("   最终 PC: {}", self.pc);
        println!("   最终栈内容: {:?}", self.stack.data);
        if let Some(result) = self.stack.peek() {
            println!("   栈顶结果: {}", result);
        }
        println!("   内存大小: {} 字节", self.memory.current_size());
        println!("   总 Gas 消耗: {} / {}", self.gas_used, self.gas_limit);
        println!("   剩余 Gas: {}", self.gas_limit - self.gas_used);
        println!(
            "   平均每步 Gas: {:.2}",
            self.gas_used as f64 / self.step_count as f64
        );
    }
}

fn main() {
    println!("🎮 EVM 完整指令序列模拟练习");
    println!("============================================================");

    // 练习 1: 简单的算术计算 (3 + 5 = 8)
    println!("\n📚 练习 1: 简单算术计算 (3 + 5)");
    println!("----------------------------------");

    let instructions = vec![
        Instruction::Push(3), // PC=0: 推入 3
        Instruction::Push(5), // PC=1: 推入 5
        Instruction::Add,     // PC=2: 执行加法
        Instruction::Stop,    // PC=3: 停止
    ];

    let mut evm = CompleteEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => evm.print_final_state(),
        Err(e) => println!("❌ 执行错误: {}", e),
    }

    // 练习 2: 复杂表达式 ((10 + 5) * 2) - 3 = 27
    println!("\n📚 练习 2: 复杂表达式 ((10 + 5) * 2) - 3");
    println!("------------------------------------------");

    let instructions = vec![
        Instruction::Push(10), // PC=0: 推入 10
        Instruction::Push(5),  // PC=1: 推入 5
        Instruction::Add,      // PC=2: 10 + 5 = 15
        Instruction::Push(2),  // PC=3: 推入 2
        Instruction::Mul,      // PC=4: 15 * 2 = 30
        Instruction::Push(3),  // PC=5: 推入 3
        Instruction::Sub,      // PC=6: 30 - 3 = 27
        Instruction::Stop,     // PC=7: 停止
    ];

    let mut evm = CompleteEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => evm.print_final_state(),
        Err(e) => println!("❌ 执行错误: {}", e),
    }

    // 练习 3: 条件跳转 (if-else 逻辑)
    println!("\n📚 练习 3: 条件跳转逻辑 (if 10 > 5 then result=100 else result=200)");
    println!("--------------------------------------------------------------------");

    let instructions = vec![
        Instruction::Push(10), // PC=0: 推入 10
        Instruction::Push(5),  // PC=1: 推入 5
        Instruction::Gt,       // PC=2: 10 > 5 ? (结果为1)
        Instruction::Push(8),  // PC=3: 推入跳转目标8
        Instruction::JumpI,    // PC=4: 如果为真跳转到PC=8
        // else 分支
        Instruction::Push(200), // PC=5: 推入 200 (else值)
        Instruction::Push(10),  // PC=6: 推入跳转目标10
        Instruction::Jump,      // PC=7: 无条件跳转到PC=10
        // if 分支
        Instruction::JumpDest,  // PC=8: 跳转目标
        Instruction::Push(100), // PC=9: 推入 100 (if值)
        // 结束
        Instruction::JumpDest, // PC=10: 跳转目标
        Instruction::Stop,     // PC=11: 停止
    ];

    let mut evm = CompleteEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => evm.print_final_state(),
        Err(e) => println!("❌ 执行错误: {}", e),
    }

    // 练习 4: 内存操作结合计算
    println!("\n📚 练习 4: 内存操作 + 计算");
    println!("------------------------------");

    let instructions = vec![
        Instruction::Push(42), // PC=0: 推入值 42
        Instruction::Push(0),  // PC=1: 推入内存偏移 0
        Instruction::MStore,   // PC=2: 存储到内存[0]
        Instruction::Push(58), // PC=3: 推入值 58
        Instruction::Push(32), // PC=4: 推入内存偏移 32
        Instruction::MStore,   // PC=5: 存储到内存[32]
        Instruction::Push(0),  // PC=6: 推入内存偏移 0
        Instruction::MLoad,    // PC=7: 从内存[0]加载 (42)
        Instruction::Push(32), // PC=8: 推入内存偏移 32
        Instruction::MLoad,    // PC=9: 从内存[32]加载 (58)
        Instruction::Add,      // PC=10: 42 + 58 = 100
        Instruction::Stop,     // PC=11: 停止
    ];

    let mut evm = CompleteEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => evm.print_final_state(),
        Err(e) => println!("❌ 执行错误: {}", e),
    }

    // 练习 5: 逻辑运算综合
    println!("\n📚 练习 5: 逻辑运算综合");
    println!("--------------------------");

    let instructions = vec![
        Instruction::Push(5), // PC=0: 推入 5
        Instruction::Push(3), // PC=1: 推入 3
        Instruction::Gt,      // PC=2: 5 > 3 = 1 (真)
        Instruction::Push(2), // PC=3: 推入 2
        Instruction::Push(4), // PC=4: 推入 4
        Instruction::Lt,      // PC=5: 2 < 4 = 1 (真)
        Instruction::And,     // PC=6: 1 & 1 = 1 (真 AND 真 = 真)
        Instruction::Stop,    // PC=7: 停止
    ];

    let mut evm = CompleteEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => evm.print_final_state(),
        Err(e) => println!("❌ 执行错误: {}", e),
    }

    // 学习总结
    println!("\n🎓 练习5学习总结:");
    println!("===========================================");
    println!("1. 完整程序执行流程：指令获取 → 解码 → 执行 → 更新状态");
    println!("2. 复杂表达式可以分解为基础指令序列");
    println!("3. 条件跳转实现了 if-else 控制流");
    println!("4. 内存操作与计算可以结合使用");
    println!("5. 逻辑运算支持复杂的布尔表达式");
    println!("6. Gas消耗模型确保程序执行的可预测性");
    println!("7. 栈机器的简洁性使得程序验证变得容易");
    println!("\n🚀 恭喜！你已经完成了EVM基础阶段的所有练习！");
}
