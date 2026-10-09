# Rust 项目模板

由 `tools/new-rust-project.ps1` 安装到新项目。

初始化器也会安装本目录的 `AGENTS.md`，确保新项目不继承参考队列试点的专属代码路径和规则。

- `engineering/INTENT.md`：目的、使用场景、成功条件与开放问题。
- `engineering/ARCHITECTURE.md`：系统边界、端到端路径、接口、状态和运行生命周期。
- `engineering/VERIFICATION_PLAN.md`：主张、范围、假设、判定器与证据登记表。
- `engineering/AI_WORK_PACKAGE.md`：每个 AI 实现任务的授权边界和完成报告。
- `engineering/REFERENCE_NOTE_TEMPLATE.md`：遇到具体问题时，记录少量参考项目的做法、适用边界与本项目取舍；它不是规格或批准。
- `assay/manifest.toml`：Assay 检查清单起点；不得把占位文本当作已声明或已验证的性质。
- `tools/prepare_work_package.py`：根据一条已接受的 `D-*.yaml` 生成 AI 工作包。

不要在初始化时预选 UI、网络、数据库或模型检查器。先完成一个真实纵切，再按其义务选择工具。
