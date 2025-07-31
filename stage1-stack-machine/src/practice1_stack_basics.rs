// 练习一：栈操作基础 - ADD 指令模拟
// 这个文件演示了如何手动模拟 EVM 的 ADD 指令执行

// 简化的栈实现（基于练习文档）
#[derive(Debug)]
struct SimpleStack {
    data: Vec<u64>, // 为了简化，使用 u64 而不是 H256
}

impl SimpleStack {
    fn new() -> Self {
        Self { data: Vec::new() }
    }
    
    fn push(&mut self, value: u64) -> Result<(), &'static str> {
        if self.data.len() >= 1000 {  // 简化的栈限制
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

// 简化的程序计数器和指令
#[derive(Debug, Clone)]
enum Instruction {
    Push(u64),    // PUSH 指令
    Add,          // ADD 指令
    Stop,         // STOP 指令
}

// 简化的 EVM 机器
#[derive(Debug)]
struct SimpleEVM {
    stack: SimpleStack,
    pc: usize,                           // 程序计数器
    instructions: Vec<Instruction>,      // 指令序列
    gas_used: u64,                      // 已使用的 Gas
}

impl SimpleEVM {
    fn new(instructions: Vec<Instruction>) -> Self {
        Self {
            stack: SimpleStack::new(),
            pc: 0,
            instructions,
            gas_used: 0,
        }
    }
    
    // 执行单条指令
    fn step(&mut self) -> Result<bool, &'static str> {
        if self.pc >= self.instructions.len() {
            return Ok(false); // 程序结束
        }
        
        let instruction = &self.instructions[self.pc].clone();
        println!("\n🔧 执行指令 [PC={}]: {:?}", self.pc, instruction);
        
        match instruction {
            Instruction::Push(value) => {
                self.stack.push(*value)?;
                self.gas_used += 3; // PUSH 指令消耗 3 gas
                self.pc += 1;
            }
            Instruction::Add => {
                // 执行 ADD 指令的详细步骤
                println!("  🧮 执行 ADD 指令:");
                
                // 1. 检查栈中是否有足够的操作数
                if self.stack.len() < 2 {
                    return Err("Stack underflow: ADD needs 2 operands");
                }
                
                // 2. 弹出两个操作数
                let operand2 = self.stack.pop()?; // 第二个操作数（栈顶）
                let operand1 = self.stack.pop()?; // 第一个操作数
                
                // 3. 执行加法运算
                let result = operand1.wrapping_add(operand2); // 使用 wrapping_add 处理溢出
                println!("     💡 计算: {} + {} = {}", operand1, operand2, result);
                
                // 4. 将结果推回栈
                self.stack.push(result)?;
                
                self.gas_used += 3; // ADD 指令消耗 3 gas
                self.pc += 1;
            }
            Instruction::Stop => {
                println!("  🛑 程序停止执行");
                return Ok(false); // 程序结束
            }
        }
        
        // 显示当前状态
        self.print_state();
        Ok(true)
    }
    
    // 运行程序直到结束
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
        println!("   已使用 Gas: {}", self.gas_used);
    }
    
    fn print_final_state(&self) {
        println!("🎯 最终状态:");
        println!("   最终栈内容: {:?}", self.stack.data);
        if let Some(result) = self.stack.peek() {
            println!("   计算结果: {}", result);
        }
        println!("   总 Gas 消耗: {}", self.gas_used);
    }
}

fn main() {
    println!("🎮 EVM 栈操作基础练习 - ADD 指令模拟");
    println!("{}", "=".repeat(50));
    
    // 练习 1: 模拟 3 + 5 的计算
    println!("\n📚 练习 1: 计算 3 + 5");
    println!("{}", "-".repeat(30));
    
    let instructions = vec![
        Instruction::Push(3),    // PUSH 3
        Instruction::Push(5),    // PUSH 5  
        Instruction::Add,        // ADD
        Instruction::Stop,       // STOP
    ];
    
    let mut evm = SimpleEVM::new(instructions);
    
    match evm.run() {
        Ok(()) => println!("✅ 练习 1 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }
    
    // 练习 2: 模拟更复杂的计算 (2 + 3) + 4 = 9
    println!("\n📚 练习 2: 计算 (2 + 3) + 4");
    println!("{}", "-".repeat(30));
    
    let instructions2 = vec![
        Instruction::Push(2),    // PUSH 2
        Instruction::Push(3),    // PUSH 3
        Instruction::Add,        // ADD (得到 5)
        Instruction::Push(4),    // PUSH 4
        Instruction::Add,        // ADD (得到 9)
        Instruction::Stop,       // STOP
    ];
    
    let mut evm2 = SimpleEVM::new(instructions2);
    
    match evm2.run() {
        Ok(()) => println!("✅ 练习 2 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }
    
    // 练习 3: 展示栈下溢错误
    println!("\n📚 练习 3: 栈下溢错误演示");
    println!("{}", "-".repeat(30));
    
    let instructions3 = vec![
        Instruction::Push(42),   // PUSH 42 (只有一个操作数)
        Instruction::Add,        // ADD (需要两个操作数，会出错)
        Instruction::Stop,
    ];
    
    let mut evm3 = SimpleEVM::new(instructions3);
    
    match evm3.run() {
        Ok(()) => println!("✅ 练习 3 完成!"),
        Err(e) => println!("❌ 预期的错误: {}", e),
    }
    
    println!("\n🎓 学习总结:");
    println!("1. EVM 使用栈来存储临时数据");
    println!("2. ADD 指令需要从栈中弹出两个操作数");
    println!("3. 计算结果会被推回栈顶");
    println!("4. 必须确保栈中有足够的操作数，否则会发生下溢");
    println!("5. 每条指令都有相应的 Gas 消耗");
}
