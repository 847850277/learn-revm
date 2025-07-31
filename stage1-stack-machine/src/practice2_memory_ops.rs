use std::collections::HashMap;

// 简化的栈实现
#[derive(Debug)]
struct SimpleStack {
    data: Vec<u64>,
}

impl SimpleStack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }

    fn push(&mut self, value: u64) -> Result<(), &'static str> {
        if self.data.len() >= 1000 {
            return Err("Stack overflow");
        }
        self.data.push(value);
        println!("  📥 PUSH: 将 {} 推入栈", value);
        println!("     栈状态: {:?}", self.data);
        Ok(())
    }

    fn pop(&mut self) -> Result<u64, &'static str> {
        match self.data.pop() {
            Some(value) => {
                println!("  📤 POP: 从栈中取出 {}", value);
                println!("     栈状态: {:?}", self.data);
                Ok(value)
            }
            None => Err("Stack underflow")
        }
    }

    fn peek(&self) -> Option<u64> {
        self.data.last().copied()
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// 简化的内存实现
#[derive(Debug)]
struct SimpleMemory {
    data: HashMap<u64, u64>, // 地址 -> 值的映射 (简化版)
    size: u64,               // 当前内存大小
}

impl SimpleMemory {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            size: 0,
        }
    }

    fn store(&mut self, offset: u64, value: u64) -> Result<u64, &'static str> {
        println!("  💾 MSTORE: 在地址 {} 存储值 {}", offset, value);

        // 计算需要的内存大小
        let required_size = offset + 32; // 每个槽 32 字节
        let old_size = self.size;

        // 如果需要扩展内存
        if required_size > self.size {
            self.size = ((required_size + 31) / 32) * 32; // 对齐到 32 字节边界
            println!("     📈 内存扩展: {} -> {} 字节", old_size, self.size);
        }

        // 存储值
        self.data.insert(offset, value);
        println!("     内存状态: {:?}", self.data);

        // 计算内存扩展的 Gas 成本
        let gas_cost = self.calculate_memory_gas(old_size, self.size);
        println!("     💰 内存 Gas 成本: {}", gas_cost);

        Ok(gas_cost)
    }

    fn load(&self, offset: u64) -> Result<u64, &'static str> {
        println!("  📖 MLOAD: 从地址 {} 加载值", offset);

        // 检查地址是否超出内存范围
        if offset >= self.size {
            println!("     ⚠️  地址超出内存范围，返回 0");
            return Ok(0);
        }

        let value = self.data.get(&offset).copied().unwrap_or(0);
        println!("     📄 加载的值: {}", value);

        Ok(value)
    }

    // 简化的内存 Gas 计算
    fn calculate_memory_gas(&self, old_size: u64, new_size: u64) -> u64 {
        if new_size <= old_size {
            return 0;
        }

        let old_words = (old_size + 31) / 32;
        let new_words = (new_size + 31) / 32;

        // 简化的二次成本模型
        let old_cost = old_words * 3 + (old_words * old_words) / 512;
        let new_cost = new_words * 3 + (new_words * new_words) / 512;

        new_cost - old_cost
    }

    fn print_memory(&self) {
        println!("     📋 内存大小: {} 字节", self.size);
        if !self.data.is_empty() {
            println!("     📋 内存内容: {:?}", self.data);
        } else {
            println!("     📋 内存内容: (空)");
        }
    }
}

// 指令类型
#[derive(Debug, Clone)]
enum Instruction {
    Push(u64),    // PUSH 指令
    MStore,       // MSTORE 指令 (offset, value) -> ()
    MLoad,        // MLOAD 指令 (offset) -> value
    Add,          // ADD 指令
    Stop,         // STOP 指令
}

// 带内存的 EVM 机器
#[derive(Debug)]
struct MemoryEVM {
    stack: SimpleStack,
    memory: SimpleMemory,
    pc: usize,
    instructions: Vec<Instruction>,
    gas_used: u64,
}

impl MemoryEVM {
    fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            stack: SimpleStack::new(),
            memory: SimpleMemory::new(),
            pc: 0,
            instructions,
            gas_used: 0,
        }
    }

    fn step(&mut self) -> Result<bool, &'static str> {
        if self.pc >= self.instructions.len() {
            return Ok(false);
        }

        let instruction = &self.instructions[self.pc].clone();
        println!("\n🔧 执行指令 [PC={}]: {:?}", self.pc, instruction);

        match instruction {
            Instruction::Push(value) => {
                self.stack.push(*value)?;
                self.gas_used += 3;
                self.pc += 1;
            }
            Instruction::MStore => {
                println!("  🧮 执行 MSTORE 指令:");

                // 检查栈中是否有足够的操作数
                if self.stack.len() < 2 {
                    return Err("Stack underflow: MSTORE needs 2 operands (offset, value)");
                }

                // 弹出操作数：offset 和 value
                let offset = self.stack.pop()?;  // 内存偏移量
                let value = self.stack.pop()?;   // 要存储的值

                // 执行内存存储
                let memory_gas = self.memory.store(offset, value)?;

                self.gas_used += 3 + memory_gas; // MSTORE 基础成本 3 + 内存扩展成本
                self.pc += 1;
            }
            Instruction::MLoad => {
                println!("  🧮 执行 MLOAD 指令:");

                // 检查栈中是否有足够的操作数
                if self.stack.len() < 1 {
                    return Err("Stack underflow: MLOAD needs 1 operand (offset)");
                }

                // 弹出偏移量
                let offset = self.stack.pop()?;

                // 从内存加载值
                let value = self.memory.load(offset)?;

                // 将值推回栈
                self.stack.push(value)?;

                self.gas_used += 3; // MLOAD 成本
                self.pc += 1;
            }
            Instruction::Add => {
                println!("  🧮 执行 ADD 指令:");

                if self.stack.len() < 2 {
                    return Err("Stack underflow: ADD needs 2 operands");
                }

                let operand2 = self.stack.pop()?;
                let operand1 = self.stack.pop()?;
                let result = operand1.wrapping_add(operand2);

                println!("     💡 计算: {} + {} = {}", operand1, operand2, result);
                self.stack.push(result)?;

                self.gas_used += 3;
                self.pc += 1;
            }
            Instruction::Stop => {
                println!("  🛑 程序停止执行");
                return Ok(false);
            }
        }

        self.print_state();
        Ok(true)
    }

    fn run(&mut self) -> Result<(), &'static str> {
        println!("🚀 开始执行 EVM 程序");
        self.print_state();

        while self.step()? {
            // 继续执行
        }

        println!("\n✅ 程序执行完成!");
        self.print_final_state();
        Ok(())
    }

    fn print_state(&self) {
        println!("📊 当前状态:");
        println!("   PC (程序计数器): {}", self.pc);
        println!("   栈内容: {:?}", self.stack.data);
        self.memory.print_memory();
        println!("   已使用 Gas: {}", self.gas_used);
    }

    fn print_final_state(&self) {
        println!("🎯 最终状态:");
        println!("   最终栈内容: {:?}", self.stack.data);
        self.memory.print_memory();
        if let Some(result) = self.stack.peek() {
            println!("   栈顶结果: {}", result);
        }
        println!("   总 Gas 消耗: {}", self.gas_used);
    }
}

fn main() {
    println!("🎮 EVM 内存操作基础练习 - MSTORE 和 MLOAD 指令模拟");
    println!("{}", "=".repeat(55));

    // 练习 1: 基本的内存存储和加载
    println!("\n📚 练习 1: 内存存储和加载");
    println!("{}", "-".repeat(30));

    let instructions1 = vec![
        Instruction::Push(42),      // PUSH 42 (要存储的值)
        Instruction::Push(0),       // PUSH 0 (内存地址)
        Instruction::MStore,        // MSTORE (在地址 0 存储值 42)
        Instruction::Push(0),       // PUSH 0 (内存地址)
        Instruction::MLoad,         // MLOAD (从地址 0 加载值)
        Instruction::Stop,          // STOP
    ];

    let mut evm1 = MemoryEVM::new(instructions1);

    match evm1.run() {
        Ok(()) => println!("✅ 练习 1 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 练习 2: 在不同地址存储多个值
    println!("\n📚 练习 2: 多地址内存操作");
    println!("{}", "-".repeat(30));

    let instructions2 = vec![
        Instruction::Push(100),     // PUSH 100 (第一个值)
        Instruction::Push(0),       // PUSH 0 (地址 0)
        Instruction::MStore,        // MSTORE
        Instruction::Push(200),     // PUSH 200 (第二个值)
        Instruction::Push(32),      // PUSH 32 (地址 32)
        Instruction::MStore,        // MSTORE
        Instruction::Push(0),       // PUSH 0
        Instruction::MLoad,         // MLOAD (加载地址 0 的值)
        Instruction::Push(32),      // PUSH 32
        Instruction::MLoad,         // MLOAD (加载地址 32 的值)
        Instruction::Add,           // ADD (100 + 200 = 300)
        Instruction::Stop,
    ];

    let mut evm2 = MemoryEVM::new(instructions2);

    match evm2.run() {
        Ok(()) => println!("✅ 练习 2 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 练习 3: 内存扩展成本演示
    println!("\n📚 练习 3: 内存扩展成本演示");
    println!("{}", "-".repeat(30));

    let instructions3 = vec![
        Instruction::Push(42),      // PUSH 42
        Instruction::Push(1000),    // PUSH 1000 (大内存地址)
        Instruction::MStore,        // MSTORE (触发大量内存扩展)
        Instruction::Stop,
    ];

    let mut evm3 = MemoryEVM::new(instructions3);

    match evm3.run() {
        Ok(()) => println!("✅ 练习 3 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    println!("\n🎓 学习总结:");
    println!("1. MSTORE 指令将栈顶两个值作为 (offset, value) 存储到内存");
    println!("2. MLOAD 指令从指定偏移量加载 32 字节数据到栈顶");
    println!("3. 内存按需扩展，扩展时需要支付额外的 Gas");
    println!("4. 内存地址必须对齐到 32 字节边界");
    println!("5. 访问超出内存范围的地址会返回 0");
    println!("6. 内存扩展的成本呈二次方增长，防止滥用");
}