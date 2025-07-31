use std::collections::HashSet;

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

// 指令类型（扩展了跳转指令）
#[derive(Debug, Clone)]
enum Instruction {
    Push(u64),      // PUSH 指令
    Add,            // ADD 指令
    Jump,           // JUMP 指令 - 无条件跳转
    JumpI,          // JUMPI 指令 - 条件跳转
    JumpDest,       // JUMPDEST 指令 - 跳转目标标记
    Stop,           // STOP 指令
}

// 跳转目标验证器
#[derive(Debug)]
struct JumpValidator {
    valid_destinations: HashSet<usize>,
}

impl JumpValidator {
    fn new(instructions: &[Instruction]) -> Self {
        let mut valid_destinations = HashSet::new();

        // 扫描所有指令，找到 JUMPDEST 的位置
        for (pc, instruction) in instructions.iter().enumerate() {
            if matches!(instruction, Instruction::JumpDest) {
                valid_destinations.insert(pc);
                println!("📍 发现有效跳转目标: PC = {}", pc);
            }
        }

        Self { valid_destinations }
    }

    fn is_valid_destination(&self, pc: usize) -> bool {
        self.valid_destinations.contains(&pc)
    }
}

// 带跳转功能的 EVM 机器
#[derive(Debug)]
struct JumpEVM {
    stack: SimpleStack,
    pc: usize,
    instructions: Vec<Instruction>,
    gas_used: u64,
    jump_validator: JumpValidator,
}

impl JumpEVM {
    fn new(instructions: Vec<Instruction>) -> Self {
        let jump_validator = JumpValidator::new(&instructions);

        Self {
            stack: SimpleStack::new(),
            pc: 0,
            instructions,
            gas_used: 0,
            jump_validator,
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
            Instruction::Jump => {
                println!("  🚀 执行 JUMP 指令:");

                // 检查栈中是否有跳转目标
                if self.stack.len() < 1 {
                    return Err("Stack underflow: JUMP needs 1 operand (destination)");
                }

                // 弹出跳转目标
                let destination = self.stack.pop()? as usize;
                println!("     🎯 跳转目标: PC = {}", destination);

                // 验证跳转目标的安全性
                if !self.jump_validator.is_valid_destination(destination) {
                    println!("     ❌ 无效跳转目标！目标 PC {} 不是 JUMPDEST", destination);
                    return Err("Invalid jump destination");
                }

                // 检查目标是否超出代码范围
                if destination >= self.instructions.len() {
                    println!("     ❌ 跳转目标超出代码范围！");
                    return Err("Jump destination out of bounds");
                }

                println!("     ✅ 跳转目标验证通过");
                self.pc = destination;
                self.gas_used += 8; // JUMP 指令成本
            }
            Instruction::JumpI => {
                println!("  🤔 执行 JUMPI 指令 (条件跳转):");

                // 检查栈中是否有足够的操作数
                if self.stack.len() < 2 {
                    return Err("Stack underflow: JUMPI needs 2 operands (destination, condition)");
                }

                // 弹出跳转目标和条件
                let destination = self.stack.pop()? as usize;
                let condition = self.stack.pop()?;

                println!("     🎯 跳转目标: PC = {}", destination);
                println!("     ❓ 跳转条件: {} ({})", condition, if condition != 0 { "真" } else { "假" });

                if condition != 0 {
                    // 条件为真，执行跳转
                    if !self.jump_validator.is_valid_destination(destination) {
                        println!("     ❌ 无效跳转目标！");
                        return Err("Invalid jump destination");
                    }

                    if destination >= self.instructions.len() {
                        println!("     ❌ 跳转目标超出代码范围！");
                        return Err("Jump destination out of bounds");
                    }

                    println!("     ✅ 条件跳转执行");
                    self.pc = destination;
                } else {
                    // 条件为假，继续顺序执行
                    println!("     ➡️ 条件为假，继续顺序执行");
                    self.pc += 1;
                }

                self.gas_used += 10; // JUMPI 指令成本
            }
            Instruction::JumpDest => {
                println!("  🏁 执行 JUMPDEST 指令:");
                println!("     📍 这是一个有效的跳转目标");

                self.gas_used += 1; // JUMPDEST 指令成本
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
        println!("🔍 跳转目标分析:");
        for dest in &self.jump_validator.valid_destinations {
            println!("   📍 有效跳转目标: PC = {}", dest);
        }

        self.print_state();

        // 防止无限循环的计数器
        let mut step_count = 0;
        const MAX_STEPS: usize = 50;

        while step_count < MAX_STEPS {
            if !self.step()? {
                break;
            }
            step_count += 1;
        }

        if step_count >= MAX_STEPS {
            println!("\n⚠️ 程序执行步数达到上限 ({})，可能存在无限循环", MAX_STEPS);
        } else {
            println!("\n✅ 程序执行完成!");
        }

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
        println!("   最终 PC: {}", self.pc);
        println!("   最终栈内容: {:?}", self.stack.data);
        if let Some(result) = self.stack.peek() {
            println!("   栈顶结果: {}", result);
        }
        println!("   总 Gas 消耗: {}", self.gas_used);
    }
}

fn main() {
    println!("🎮 EVM 跳转指令安全练习 - JUMP 和 JUMPDEST 指令模拟");
    println!("{}", "=".repeat(60));

    // 练习 1: 基本的无条件跳转
    println!("\n📚 练习 1: 基本无条件跳转");
    println!("{}", "-".repeat(30));

    let instructions1 = vec![
        Instruction::Push(5),       // PC=0: PUSH 5
        Instruction::Jump,          // PC=1: JUMP (跳转到 PC=5)
        Instruction::Push(99),      // PC=2: PUSH 99 (这条指令会被跳过)
        Instruction::Add,           // PC=3: ADD (这条指令会被跳过)
        Instruction::Stop,          // PC=4: STOP (这条指令会被跳过)
        Instruction::JumpDest,      // PC=5: JUMPDEST (跳转目标)
        Instruction::Push(42),      // PC=6: PUSH 42
        Instruction::Stop,          // PC=7: STOP
    ];

    let mut evm1 = JumpEVM::new(instructions1);

    match evm1.run() {
        Ok(()) => println!("✅ 练习 1 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 练习 2: 条件跳转演示
    println!("\n📚 练习 2: 条件跳转 (JUMPI)");
    println!("{}", "-".repeat(30));

    let instructions2 = vec![
        Instruction::Push(1),       // PC=0: PUSH 1 (条件为真)
        Instruction::Push(6),       // PC=1: PUSH 6 (跳转目标)
        Instruction::JumpI,         // PC=2: JUMPI (条件跳转)
        Instruction::Push(100),     // PC=3: PUSH 100 (会被跳过)
        Instruction::Stop,          // PC=4: STOP (会被跳过)
        Instruction::Push(200),     // PC=5: PUSH 200 (会被跳过)
        Instruction::JumpDest,      // PC=6: JUMPDEST (跳转目标)
        Instruction::Push(300),     // PC=7: PUSH 300
        Instruction::Stop,          // PC=8: STOP
    ];

    let mut evm2 = JumpEVM::new(instructions2);

    match evm2.run() {
        Ok(()) => println!("✅ 练习 2 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 练习 3: 条件跳转 - 条件为假
    println!("\n📚 练习 3: 条件跳转 - 条件为假");
    println!("{}", "-".repeat(30));

    let instructions3 = vec![
        Instruction::Push(0),       // PC=0: PUSH 0 (条件为假)
        Instruction::Push(6),       // PC=1: PUSH 6 (跳转目标)
        Instruction::JumpI,         // PC=2: JUMPI (条件跳转，不会跳转)
        Instruction::Push(100),     // PC=3: PUSH 100 (会被执行)
        Instruction::Stop,          // PC=4: STOP
        Instruction::Push(200),     // PC=5: PUSH 200 (不会被执行)
        Instruction::JumpDest,      // PC=6: JUMPDEST (跳转目标)
        Instruction::Push(300),     // PC=7: PUSH 300 (不会被执行)
        Instruction::Stop,          // PC=8: STOP (不会被执行)
    ];

    let mut evm3 = JumpEVM::new(instructions3);

    match evm3.run() {
        Ok(()) => println!("✅ 练习 3 完成!"),
        Err(e) => println!("❌ 错误: {}", e),
    }

    // 练习 4: 无效跳转演示
    println!("\n📚 练习 4: 无效跳转错误演示");
    println!("{}", "-".repeat(30));

    let instructions4 = vec![
        Instruction::Push(3),       // PC=0: PUSH 3 (无效跳转目标)
        Instruction::Jump,          // PC=1: JUMP (尝试跳转到 PC=3)
        Instruction::Stop,          // PC=2: STOP
        Instruction::Push(42),      // PC=3: PUSH 42 (不是 JUMPDEST!)
        Instruction::Stop,          // PC=4: STOP
    ];

    let mut evm4 = JumpEVM::new(instructions4);

    match evm4.run() {
        Ok(()) => println!("✅ 练习 4 完成!"),
        Err(e) => println!("❌ 预期的错误: {}", e),
    }

    println!("\n🎓 学习总结:");
    println!("1. JUMP 指令实现无条件跳转，需要栈顶提供目标地址");
    println!("2. JUMPI 指令实现条件跳转，需要目标地址和条件值");
    println!("3. 只能跳转到 JUMPDEST 指令标记的位置，保证安全性");
    println!("4. 跳转目标在程序启动前预先验证和缓存");
    println!("5. 无效跳转会立即终止程序执行，防止恶意代码");
    println!("6. 条件跳转根据栈顶值决定是否跳转 (0=假, 非0=真)");
    println!("7. Gas 成本: JUMP=8, JUMPI=10, JUMPDEST=1");
}