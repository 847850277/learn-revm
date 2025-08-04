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

// 简化的内存实现（用于 Gas 计算）
#[derive(Debug, Clone)]
struct SimpleMemory {
    data: HashMap<u64, u64>,
    size: u64, // 当前内存大小（字节）
}

impl SimpleMemory {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
            size: 0,
        }
    }

    fn expand_to(&mut self, new_size: u64) -> u64 {
        let old_size = self.size;
        if new_size > self.size {
            self.size = new_size;
            // 对齐到 32 字节边界
            let aligned_size = (new_size + 31) / 32 * 32;
            self.size = aligned_size;
        }
        self.calculate_memory_expansion_gas(old_size, self.size)
    }

    // 内存扩展 Gas 计算（简化版本）
    fn calculate_memory_expansion_gas(&self, old_size: u64, new_size: u64) -> u64 {
        if new_size <= old_size {
            return 0;
        }

        let old_words = (old_size + 31) / 32;
        let new_words = (new_size + 31) / 32;

        let old_cost = self.memory_cost(old_words);
        let new_cost = self.memory_cost(new_words);

        new_cost - old_cost
    }

    // 内存成本计算（二次方增长）
    fn memory_cost(&self, words: u64) -> u64 {
        let linear_cost = words * 3;
        let quadratic_cost = words * words / 512;
        linear_cost + quadratic_cost
    }

    fn store(&mut self, offset: u64, value: u64) -> Result<u64, &'static str> {
        // 计算需要的内存大小
        let required_size = offset + 32;
        let expansion_gas = self.expand_to(required_size);

        self.data.insert(offset, value);
        Ok(expansion_gas)
    }

    fn load(&self, offset: u64) -> Result<(u64, u64), &'static str> {
        // 即使是读取也可能触发内存扩展
        let required_size = offset + 32;
        let expansion_gas = if required_size > self.size {
            // 这里应该扩展内存，但为了简化只计算Gas
            self.calculate_memory_expansion_gas(self.size, required_size)
        } else {
            0
        };

        let value = self.data.get(&offset).copied().unwrap_or(0);
        Ok((value, expansion_gas))
    }

    fn current_size(&self) -> u64 {
        self.size
    }
}

// 存储模拟（用于 SLOAD/SSTORE Gas 计算）
#[derive(Debug, Clone)]
struct SimpleStorage {
    data: HashMap<u64, u64>,
}

impl SimpleStorage {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn sload(&self, key: u64) -> (u64, u64) {
        let value = self.data.get(&key).copied().unwrap_or(0);
        let gas_cost = 200; // SLOAD 基础成本
        (value, gas_cost)
    }

    fn sstore(&mut self, key: u64, value: u64) -> u64 {
        let current_value = self.data.get(&key).copied().unwrap_or(0);

        let gas_cost = if current_value == 0 && value != 0 {
            // 从零设置为非零值
            20000
        } else if current_value != 0 && value == 0 {
            // 从非零设置为零值（有退款，但这里简化）
            5000
        } else if current_value != 0 && value != 0 {
            // 修改非零值
            5000
        } else {
            // 从零设置为零（无操作）
            200
        };

        self.data.insert(key, value);
        gas_cost
    }
}

// Gas 计算指令枚举
#[derive(Debug, Clone)]
enum Instruction {
    // 基础算术指令
    Push(u64),
    Add,
    Mul,
    Sub,

    // 内存指令
    MStore, // 存储到内存
    MLoad,  // 从内存加载

    // 存储指令
    SLoad,  // 从存储加载
    SStore, // 存储到存储

    // 控制指令
    Stop,
}

// Gas 感知的 EVM
#[derive(Debug)]
struct GasEVM {
    stack: SimpleStack,
    memory: SimpleMemory,
    storage: SimpleStorage,
    instructions: Vec<Instruction>,
    pc: usize,
    gas_used: u64,
    gas_limit: u64,
}

impl GasEVM {
    fn new(instructions: Vec<Instruction>, gas_limit: u64) -> Self {
        Self {
            stack: SimpleStack::new(),
            memory: SimpleMemory::new(),
            storage: SimpleStorage::new(),
            instructions,
            pc: 0,
            gas_used: 0,
            gas_limit,
        }
    }

    fn check_gas(&self, required_gas: u64) -> Result<(), &'static str> {
        if self.gas_used + required_gas > self.gas_limit {
            return Err("Out of gas");
        }
        Ok(())
    }

    fn consume_gas(&mut self, gas: u64) -> Result<(), &'static str> {
        self.check_gas(gas)?;
        self.gas_used += gas;
        println!("     💰 消耗 Gas: {} (总计: {})", gas, self.gas_used);
        Ok(())
    }

    fn step(&mut self) -> Result<bool, &'static str> {
        if self.pc >= self.instructions.len() {
            return Ok(false);
        }

        let instruction = self.instructions[self.pc].clone();
        println!("\n🔧 执行指令 [PC={}]: {:?}", self.pc, instruction);

        match instruction {
            Instruction::Push(value) => {
                self.consume_gas(3)?; // PUSH 指令基础成本
                self.stack.push(value)?;
                println!("  📥 PUSH: 将 {} 推入栈", value);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::Add => {
                self.consume_gas(3)?; // ADD 指令成本
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a + b;
                self.stack.push(result)?;
                println!("  ➕ ADD: {} + {} = {}", a, b, result);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::Mul => {
                self.consume_gas(5)?; // MUL 指令成本
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a * b;
                self.stack.push(result)?;
                println!("  ✖️ MUL: {} * {} = {}", a, b, result);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::Sub => {
                self.consume_gas(3)?; // SUB 指令成本
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let result = a.saturating_sub(b);
                self.stack.push(result)?;
                println!("  ➖ SUB: {} - {} = {}", a, b, result);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::MStore => {
                self.consume_gas(3)?; // MSTORE 基础成本
                let offset = self.stack.pop()?;
                let value = self.stack.pop()?;

                let expansion_gas = self.memory.store(offset, value)?;
                if expansion_gas > 0 {
                    self.consume_gas(expansion_gas)?;
                    println!("     💾 内存扩展成本: {} gas", expansion_gas);
                }

                println!("  💾 MSTORE: 在偏移 {} 存储值 {}", offset, value);
                println!("     内存大小: {} 字节", self.memory.current_size());
                self.pc += 1;
            }

            Instruction::MLoad => {
                self.consume_gas(3)?; // MLOAD 基础成本
                let offset = self.stack.pop()?;

                let (value, expansion_gas) = self.memory.load(offset)?;
                if expansion_gas > 0 {
                    self.consume_gas(expansion_gas)?;
                    println!("     💾 内存扩展成本: {} gas", expansion_gas);
                }

                self.stack.push(value)?;
                println!("  💾 MLOAD: 从偏移 {} 加载值 {}", offset, value);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::SLoad => {
                let key = self.stack.pop()?;
                let (value, gas_cost) = self.storage.sload(key);
                self.consume_gas(gas_cost)?;
                self.stack.push(value)?;
                println!("  🗄️ SLOAD: 从槽 {} 加载值 {}", key, value);
                println!("     栈状态: {:?}", self.stack.data);
                self.pc += 1;
            }

            Instruction::SStore => {
                let key = self.stack.pop()?;
                let value = self.stack.pop()?;
                let gas_cost = self.storage.sstore(key, value);
                self.consume_gas(gas_cost)?;
                println!("  🗄️ SSTORE: 在槽 {} 存储值 {}", key, value);
                self.pc += 1;
            }

            Instruction::Stop => {
                println!("  🛑 程序停止执行");
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn run(&mut self) -> Result<(), &'static str> {
        println!("🚀 开始执行 EVM 程序 (Gas 限制: {})", self.gas_limit);

        while self.step()? {
            self.print_state();
        }

        Ok(())
    }

    fn print_state(&self) {
        println!("📊 当前状态:");
        println!("   PC (程序计数器): {}", self.pc);
        println!("   栈内容: {:?}", self.stack.data);
        println!("   内存大小: {} 字节", self.memory.current_size());
        println!("   已使用 Gas: {} / {}", self.gas_used, self.gas_limit);
    }

    fn print_final_state(&self) {
        println!("\n🎯 最终状态:");
        println!("   最终 PC: {}", self.pc);
        println!("   最终栈内容: {:?}", self.stack.data);
        if let Some(top) = self.stack.peek() {
            println!("   栈顶结果: {}", top);
        }
        println!("   内存大小: {} 字节", self.memory.current_size());
        println!("   总 Gas 消耗: {} / {}", self.gas_used, self.gas_limit);
        println!("   剩余 Gas: {}", self.gas_limit - self.gas_used);
    }
}

fn main() {
    println!("🎮 EVM Gas 计算基础练习 - 理解资源消耗机制");
    println!("============================================================");

    // 练习 1: 基础指令的 Gas 消耗
    println!("\n📚 练习 1: 基础算术指令 Gas 消耗");
    println!("------------------------------");

    let instructions = vec![
        Instruction::Push(10), // 3 gas
        Instruction::Push(20), // 3 gas
        Instruction::Add,      // 3 gas
        Instruction::Push(5),  // 3 gas
        Instruction::Mul,      // 5 gas
        Instruction::Stop,     // 0 gas
    ];

    let mut evm = GasEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => {
            println!("✅ 程序执行完成!");
            evm.print_final_state();
        }
        Err(e) => println!("❌ 执行错误: {}", e),
    }
    println!("✅ 练习 1 完成!");

    // 练习 2: 内存操作的 Gas 成本
    println!("\n📚 练习 2: 内存操作 Gas 成本");
    println!("------------------------------");

    let instructions = vec![
        Instruction::Push(100), // 要存储的值
        Instruction::Push(0),   // 内存偏移 0
        Instruction::MStore,    // 存储到内存，触发内存扩展
        Instruction::Push(200), // 要存储的值
        Instruction::Push(64),  // 内存偏移 64
        Instruction::MStore,    // 再次扩展内存
        Instruction::Push(0),   // 从偏移 0 加载
        Instruction::MLoad,     // 加载值
        Instruction::Stop,
    ];

    let mut evm = GasEVM::new(instructions, 1000);
    match evm.run() {
        Ok(()) => {
            println!("✅ 程序执行完成!");
            evm.print_final_state();
        }
        Err(e) => println!("❌ 执行错误: {}", e),
    }
    println!("✅ 练习 2 完成!");

    // 练习 3: 存储操作的高 Gas 成本
    println!("\n📚 练习 3: 存储操作高 Gas 成本");
    println!("------------------------------");

    let instructions = vec![
        Instruction::Push(42),  // 要存储的值
        Instruction::Push(1),   // 存储槽 1
        Instruction::SStore,    // 第一次存储（从零到非零，20000 gas）
        Instruction::Push(1),   // 存储槽 1
        Instruction::SLoad,     // 读取存储（200 gas）
        Instruction::Push(100), // 新值
        Instruction::Push(1),   // 存储槽 1
        Instruction::SStore,    // 修改存储（5000 gas）
        Instruction::Stop,
    ];

    let mut evm = GasEVM::new(instructions, 30000); // 需要更多 gas
    match evm.run() {
        Ok(()) => {
            println!("✅ 程序执行完成!");
            evm.print_final_state();
        }
        Err(e) => println!("❌ 执行错误: {}", e),
    }
    println!("✅ 练习 3 完成!");

    // 练习 4: Gas 不足错误演示
    println!("\n📚 练习 4: Gas 不足错误演示");
    println!("------------------------------");

    let instructions = vec![
        Instruction::Push(42),
        Instruction::Push(1),
        Instruction::SStore, // 需要 20000+ gas，但我们只给 1000
        Instruction::Stop,
    ];

    let mut evm = GasEVM::new(instructions, 1000); // 故意设置低 gas 限制
    match evm.run() {
        Ok(()) => {
            println!("✅ 程序执行完成!");
            evm.print_final_state();
        }
        Err(e) => println!("❌ 预期的错误: {}", e),
    }

    // 学习总结
    println!("\n🎓 学习总结:");
    println!("1. 基础算术指令 Gas 成本较低 (ADD=3, MUL=5)");
    println!("2. 内存操作会触发内存扩展，成本随内存大小二次方增长");
    println!("3. 存储操作成本很高 (SSTORE=5000-20000, SLOAD=200)");
    println!("4. Gas 限制防止无限循环和资源滥用");
    println!("5. 不同操作的 Gas 成本反映了它们的计算复杂度");
    println!("6. 内存扩展采用二次方定价防止内存滥用");
    println!("7. 存储操作昂贵是因为需要永久保存在区块链上");
}
