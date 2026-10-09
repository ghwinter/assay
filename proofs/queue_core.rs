// proofs/queue_core.rs — Verus 机器证明入口（O-021→O-028，D-006/D-017）
//
// D-017 起本文件不再是独立的证明对象，而是**验证入口**：include 进来的
// pilot-rs/src/qcore.rs 本体就是被证明与被编译的同一份代码（单一事实源）——
// 契约以 cfg_attr(verus_verify, verus_spec(...)) 形式住在实现文件里，
// 普通 cargo 看不到它们，Verus 读取它们。
//
// 分工边界（检查器阶梯，不变）：
//   顺序核心正确性 = qcore.rs（本入口 include 的文件，O-028 证明档）
//   并发协议（阻塞/唤醒/close 时序）= TLA+ 档（tla/queue.tla，O-016/O-017）
//   数据竞争自由 = 语言强制档（O-011，safe Rust 零 unsafe）
//
// 运行：verus.exe --cfg verus_verify proofs/queue_core.rs
//       （路径见 tools/config.toml [tools].verus；工具链需 1.98.1）

include!("../pilot-rs/src/qcore.rs");
