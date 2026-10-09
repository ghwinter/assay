# Verus 检查档工作流（实现层：机器证明）

定位：证明**顺序核心**——数据结构不变量、函数前后置条件、纯逻辑性质。
不建模阻塞/并发时序（那是 TLA+ 档的职责）；数据竞争自由由 safe Rust 语言强制档兜底。
何时用：核心数据结构或关键纯逻辑需要机器证明级证据时。

## 单一事实源（D-017 起）

证明对象就是实现本体：契约以 `#[cfg_attr(verus_verify, verus_spec(...))]`
住在 `pilot-rs/src/qcore.rs` 里，spec 模块以 `#[cfg(verus_verify)]` 门控。

- `cargo build/test`：契约不存在，纯净零依赖（执行门 2 不破）
- `verus --cfg verus_verify proofs/queue_core.rs`：include 实现文件，契约生效
- 修改核心逻辑 = 修改同一份文件，证明与编译永不脱钩

## 依赖清单

| 依赖 | 版本/位置 | 说明 |
|---|---|---|
| verus 发布版 | 见 `tools/config.toml [tools].verus`（0.2026.09.27-x86-win，含 z3.exe 与 vstd.vir） | **仓库外**，路径单点收敛于 config；升级后 proofs 全量重跑 |
| rustup 工具链 | 1.98.1-x86_64-pc-windows-msvc | verus.exe 运行必需（版本不符会明确报错）；`rustup toolchain install 1.98.1-x86_64-pc-windows-msvc` |
| MSVC 链接环境 | VS Build Tools（机器已有） | rustc msvc 目标 |
| 无第三方 crate | proofs 零依赖 | vstd/verus_spec 由 verus.exe 注入；普通 cargo 侧契约全 cfg 门控 |

接入新项目：拷贝 `proofs/`（入口 include 模式）+ 在实现 crate 建 `<core>.rs`
（参照 qcore.rs 的契约门控模式）+ 确认上述机器依赖。

## 流程（六步）

1. **提取核心**：把要证明的逻辑收敛为 owned 值传递的纯函数/结构
   （取/还值，不用 &mut self——见坑 2），放实现 crate 的 `<core>.rs`。
2. **写规约**：cfg 门控的 spec 模块（`external_type_specification` 桥 + `inv`）
   + 每个函数 `#[cfg_attr(verus_verify, verus_spec(res => requires..., ensures...))]`。
   **规约必须完整刻画结果**：构造函数写全字段等式（缺 `q.cap == cap` 曾致断链），
   字段保持条款（cap 不变）单独成句。
3. **验证用例**：cfg 门控的 `verus!{ fn main }` 内以断言机器检验行为序列
   （如 FIFO：入 11,12 → 拒 → 出 11,12 → None）。
4. **运行**：`verus.exe --cfg verus_verify proofs/<name>.rs`（路径读 config）
   → 目标输出 `N verified, 0 errors`；同文件 `cargo test` 必须照常全绿。
5. **入账**：结果与命令写入 `ledger/evidence/O-xxx.yaml`。
6. **升级纪律**：规约改动后全量重跑 proofs；verus 升级后同上。

## 坑清单（2026-10 实战沉淀）

1. **new-mut-ref 消歧**：ensures 中 &mut 参数必须 `old(x)`（入场值）/`final(x)`（终值）；
   原始类型还需显式解引用 `*old(v)`。
2. **&mut self 多次调用链有调用方传播缺口**（本版 SMT 实例化问题：单事实可证、组合
   不可证）——核心证明用 owned 值传递（`fn push(mut q: QueueCore, v) -> (res: (QueueCore, bool))`）规避。
3. **规约完备性**：漏写字段等式/保持条款 = 调用链知识断链（new 与 push/pop 各踩一次）。
4. exec 函数调用不能放进 assert 表达式——先绑定变量再断言。
5. rust 工具链必须恰为 1.98.1；stable 1.99 会被 verus.exe 拒绝。
6. 发布包获取：GitHub API 匿名限流、直链猜名 404——从 releases 页的
   `expanded_assets/release/<tag>` 拿准确资产名（形如 `verus-<tag>-x86-win.zip`，
   **tag 自带 release/ 前缀**，下载 URL 也要带）。
7. 契约门控模式（D-017）：`//!` 内文档注释在 include! 场景非法（E0753，改 `//`）；
   `mut self` 不被支持（owned 值传递规避）；`mem::take` 需 `impl Default`；
   `--cfg` 由 verus 驱动透传给 rustc，普通 rustc 对未启用的 `cfg_attr` 契约零告警。

## 证据语义边界

通过 = 规约在被验证函数上的机器证明。它**不**覆盖并发交互（无阻塞建模）——
协议层归 TLA+；也**不**覆盖规约与真实意图的一致（那是 G 账本与人的裁决）。
