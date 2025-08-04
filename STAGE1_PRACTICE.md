# 🎮 第一阶段实践练习：手把手理解 EVM 执行

## 🎯 练习目标
通过模拟 EVM 指令执行，深入理解栈机器的工作原理

---

## 📚 练习 1：栈操作基础

### 🔢 模拟 ADD 指令执行

**初始状态**：
```
栈: [0x05, 0x03]  (栈顶是 0x03)
程序计数器: 0
指令: ADD (0x01)
```

**执行步骤**：
1. **取指令**: 从 code[0] 获取操作码 0x01 (ADD)
2. **解码**: 识别为二元算术运算
3. **取操作数**:
   - pop(): 取得 0x03 (第一个操作数)
   - pop(): 取得 0x05 (第二个操作数)
4. **执行运算**: 0x05 + 0x03 = 0x08
5. **存储结果**: push(0x08)
6. **更新PC**: PC = PC + 1

**最终状态**：
```
栈: [0x08]
程序计数器: 1
```

### 💻 Rust 代码对应

让我们看看实际的代码是如何实现的：

```rust
fn eval_add(state: &mut Machine) -> Control {
    op2_u256_tuple!(state, overflowing_add)
}
```

宏展开后相当于：
```rust
fn eval_add(state: &mut Machine) -> Control {
    // 1. 从栈中弹出两个 U256 值
    let op1 = match state.stack.pop() {
        Ok(value) => U256::from_big_endian(&value[..]),
        Err(e) => return Control::Exit(e.into()),
    };
    let op2 = match state.stack.pop() {
        Ok(value) => U256::from_big_endian(&value[..]),
        Err(e) => return Control::Exit(e.into()),
    };

    // 2. 执行溢出安全的加法
    let (ret, _overflow) = op1.overflowing_add(op2);

    // 3. 将结果推回栈
    let mut value = H256::default();
    ret.to_big_endian(&mut value[..]);
    match state.stack.push(value) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    }

    // 4. 继续执行下一条指令
    Control::Continue(1)
}
```

---

## 📚 练习 2：内存操作理解

### 💾 模拟 MSTORE 指令

**初始状态**：
```
栈: [0x20, 0x1234...] (栈顶是要存储的值)
内存: [] (空)
```

**MSTORE 指令功能**：将栈顶的 32 字节值存储到指定内存位置

**执行过程**：
1. `pop()`: 获取内存偏移量 0x20 (32)
2. `pop()`: 获取要存储的值 0x1234...
3. 检查内存边界：需要从偏移 32 开始存储 32 字节
4. 内存扩展：如果当前内存小于 64 字节，扩展到 64 字节
5. 写入内存：将 32 字节值写入内存[32:64]

**内存扩展计算**：
```
需要内存大小 = offset + 32 = 32 + 32 = 64 字节
对齐到 32 字节边界 = next_multiple_of_32(64) = 64 字节
```

---

## 📚 练习 3：跳转指令安全检查

### 🚀 JUMP 指令的安全机制

EVM 的 JUMP 指令不能随意跳转，只能跳转到 JUMPDEST 标记的位置。

**字节码示例**：
```
0x00: PUSH1 0x06    // 将跳转目标 6 推入栈
0x02: JUMP          // 跳转到栈顶指定的位置
0x03: STOP          // 这条指令会被跳过
0x04: STOP          // 这条指令会被跳过
0x05: STOP          // 这条指令会被跳过
0x06: JUMPDEST      // 有效的跳转目标
0x07: STOP          // 跳转后执行这里
```

**安全检查过程**：
1. 从栈中弹出跳转目标：6
2. 检查 `valid_jump_addresses.is_valid(6)`
3. 如果位置 6 是 JUMPDEST，允许跳转
4. 否则返回 `ExitError::InvalidJump`

**`ValidJumpAddress` 的构建**：
```rust
pub fn new(code: &[u8]) -> Self {
    let mut jumps: Vec<bool> = Vec::with_capacity(code.len());
    jumps.resize(code.len(), false);

    let mut i = 0;
    while i < code.len() {
        let opcode = code[i] as u8;
        if opcode == OpCode::JUMPDEST as u8 {
            jumps[i] = true;  // 标记为有效跳转点
            i += 1;
        } else if let Some(v) = OpCode::is_push(opcode) {
            i += v as usize + 1;  // 跳过 PUSH 指令的数据部分
        } else {
            i += 1;
        }
    }

    Self(jumps)
}
```

---

## 📚 练习 4：Gas 计算基础

### ⛽ 理解 Gas 的作用

Gas 是 EVM 执行的"燃料"，防止无限循环和资源滥用。

**基础 Gas 成本**：
- ADD: 3 gas
- MUL: 5 gas
- SLOAD: 200 gas (读取存储)
- SSTORE: 5000/20000 gas (写入存储)

**内存扩展 Gas**：
内存使用越多，扩展成本越高（二次方增长）

```rust
// 简化的内存 gas 计算
fn memory_gas(new_size: usize) -> u64 {
    let new_size = (new_size + 31) / 32;  // 转换为字数
    let new_size_squared = new_size * new_size;
    (new_size * 3) + (new_size_squared / 512)
}
```

---

## 🎯 练习 5：完整指令序列模拟

### 🔢 计算 3 + 5 的完整过程

**智能合约字节码**：
```
0x00: PUSH1 0x03    // 将 3 推入栈
0x02: PUSH1 0x05    // 将 5 推入栈
0x04: ADD           // 执行加法
0x05: STOP          // 停止执行
```

**详细执行过程**：

| 步骤 | PC | 指令 | 栈状态 | Gas 使用 | 说明 |
|------|----|----- |--------|----------|------|
| 0 | 0 | PUSH1 0x03 | [0x03] | 3 | 推入常量 3 |
| 1 | 2 | PUSH1 0x05 | [0x03, 0x05] | 3 | 推入常量 5 |
| 2 | 4 | ADD | [0x08] | 3 | 执行 3+5=8 |
| 3 | 5 | STOP | [0x08] | 0 | 停止执行 |

**总 Gas 消耗**: 9 gas

---

## 🧠 深度思考题

### 1. 为什么 EVM 选择栈机器而不是寄存器机器？

**栈机器优势**：
- 实现简单，无需寄存器分配
- 指令编码紧凑（无需指定寄存器）
- 验证容易，状态确定性强

**寄存器机器优势**：
- 执行效率更高（减少栈操作）
- 编译器优化空间大

**EVM 选择栈机器的原因**：
- 简化虚拟机实现
- 便于形式化验证
- 减少共识复杂性

### 2. 内存为什么要 32 字节对齐？

- **EVM 字长**：EVM 是 256 位机器，32 字节 = 256 位
- **哈希友好**：Keccak256 输出 32 字节
- **性能优化**：对齐访问通常更快
- **规范统一**：简化内存操作的复杂性

### 3. 如何防止 Gas 攻击？

- **预付费模式**：执行前检查 Gas 是否足够
- **动态计价**：复杂操作消耗更多 Gas
- **上限保护**：区块 Gas limit 防止无限循环

---

## ✅ 练习检查点

完成以上练习后，您应该能够：

- [ ] 手动模拟基础指令的执行过程
- [ ] 理解栈操作的安全检查机制
- [ ] 解释内存扩展的对齐规则
- [ ] 分析跳转指令的安全验证
- [ ] 计算简单指令序列的 Gas 消耗

**下一步**：准备进入第二阶段，学习规范化架构重构！

---

**学习提示**：可以在 VS Code 中打开相关源文件，对照代码理解这些概念。真正的理解来自于看到代码如何实现这些抽象概念！
