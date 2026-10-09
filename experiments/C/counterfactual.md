# 实验 C：反事实审计——agent 分类输出（被审计物）

反事实命题：「若 D-009 改为 lock-free ring（取代 mutex+双 condvar），账本中哪些实体仍成立？」

日期：2026-10-05。分类者：AI（待人工金标准校验）。
五分类定义：
- still-valid：实体及其检查/证据在反事实下仍然成立或只需重测
- invalidated：实体被反事实推翻（本体消失或必须重写）
- newly-required：反事实新产生的义务（原账本未覆盖）
- unaffected：与并发策略无关（归档/通用工具）
- unknown：无法判断

统计：still-valid ×24，invalidated ×5，unaffected ×20，newly-required ×4（见下）。

## 决策（D-001..D-011）

| id | 分类 | 理由 |
|---|---|---|
| D-001 | unaffected | 已 superseded 的历史 |
| D-002..D-005 | unaffected | C++ 归档（D-008 只读） |
| D-006 | still-valid | 检查器档位与锁策略正交（但 tla 模型本体需重写，见工件层） |
| D-007 | still-valid | 主语言不变 |
| D-008 | unaffected | 归档决策 |
| D-009 | invalidated | 反事实直接替换的对象 |
| D-010 | still-valid | 固定容量与 move 语义在 ring 下不变 |
| D-011 | still-valid | 诊断档与锁策略无关 |

## 义务（O-001..O-023）

| id | 分类 | 理由 |
|---|---|---|
| O-001..O-010 | unaffected | C++ 归档的历史义务 |
| O-011 竞争自由 | still-valid | safe Rust + atomics 仍无数据竞争；属性保持 |
| O-012 死锁自由 | still-valid | 无锁构造下平凡成立（注意：其"单锁 grep"检查器失效——失效在证据层） |
| O-013 不变量 | still-valid | ring 仍有 0<=size<=cap |
| O-014 保序 | still-valid | FIFO 仍是契约 |
| O-015 吞吐预算 | still-valid | 证据需重测 |
| O-016 唤醒正确性 | invalidated | condvar 等待语义随协议消失，由 newly-required 的通知/内存序义务替代 |
| O-017 关闭语义 | still-valid | close/排空语义必须保留；检查器需改写 |
| O-018 背压 | still-valid | 固定容量不变 |
| O-019 所有权边界 | still-valid | 语言强制不变 |
| O-020 panic 安全 | still-valid | 不变 |
| O-021 Verus 核心 | still-valid | **顺序核心与并发策略正交——证明原样存活** |
| O-022/O-023 诊断 | still-valid | 不变 |

## 工件

| 路径 | 分类 | 理由 |
|---|---|---|
| pilot-rs/src/lib.rs | invalidated | 实现整体替换 |
| pilot-rs/tests/queue.rs | invalidated | condvar 特定用例失效；黑盒断言可迁移 |
| tla/queue.tla | invalidated | 协议模型必须重写（无阻塞状态机 + 内存序） |
| proofs/queue_core.rs | still-valid | 原样存活 |
| tools/* | unaffected | 与实现无关 |
| triggers/* | still-valid | 触发器仍适用（blocking 触发器在"保留阻塞语义"前提下成立） |

## newly-required（反事实新产生的义务）

1. **内存序正确性**：acquire/release 语义的形式化或系统化测试——lock-free 的正确性核心
2. **进度保证**：lock-freedom / 无饥饿的形式化边界
3. **ABA / 索引回绕**：head/tail 回绕正确性
4. **阻塞语义重建**：若须保留 G-001/G-002 的阻塞 push/pop，需要新的通知机制（事件计数/回退）及其义务
