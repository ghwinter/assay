# 触发器候选池（挖矿 v1，未激活）

来源：NPR 7150.2B（NASA 软件工程需求）、SWEBOK V4（18 知识域）、
Arcadia 五视角，对齐后端引擎领域。**本文件是候选池不是触发器库**——
激活任一条需走 D 裁决（追加进 ledger/triggers/*.yaml 并过触发器覆盖检查）。
现有库覆盖 Requirements/Architecture/Design/Construction/Testing 五域
（检查器阶梯 + 10 条规则）；本池补的是 **Operations / CM / Security /
Maintenance 四个弱域**（恰为 SWEBOK V4 新升格的三个 KA 所在）。

## 候选触发器组

### adds_persistence（引入持久化：数据库/文件/WAL）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| durability_semantics | 写路径持久化点（fsync 策略）与崩溃后可恢复性显式声明 | test（kill -9 恢复）/ TLA+ | SWE-052/065 |
| schema_migration | schema 变更有迁移路径与回滚 | test + lint（迁移登记） | SWE-079-085（CM） |
| data_consistency | 崩溃/重启后不变量保持 | test（崩溃注入）/ TLA+ | SWE-066 |

### adds_network_io（引入网络调用/服务间通信）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| partial_failure | 每个远程调用的失败语义（超时/重试/幂等）显式声明 | test（故障注入）+ lint | WORKFLOW 已列 |
| timeout_budget | 超时与重试预算有下游锚点（BS-001） | bench/config lint | 本仓库先例 G-003 |
| protocol_versioning | 接口版本兼容策略成文 | test（兼容性） | SWE-055 |

### adds_background_task（后台任务/定时/异步作业）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| task_lifecycle | 任务启停/崩溃重启语义 | test | SWEBOK Operations |
| shutdown_draining | 关停时在途任务排空策略 | test | 本仓库先例 O-017 |
| idempotency | 重复执行安全 | property test | SWE-066 |

### adds_config_surface（配置/热加载）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| config_validation | 非法配置的失败模式（启动失败 vs 回退）成文 | test | SWE-050（需求可检验化） |
| hot_reload_consistency | 热加载一致性窗口 | test / TLA+（状态机） | Arcadia 行为建模 |

### adds_public_api（公共 API/插件接口）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| api_compatibility | 破坏性变更有版本化记录 | lint（semver）/ test | SWE-055、SWEBOK Architecture |
| input_validation | 边界输入语义（空/超界/恶意） | property test | SWE-060s |
| plugin_isolation | 插件故障不拖垮宿主 | test（故障隔离） | SWE-023（安全关键防护） |

### adds_release_channel（构建/发布/多平台）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| build_reproducibility | 构建可复现（锁文件、版本固定、离线可建） | lint / CI | SWE-085（CM 存储/交付） |
| artifact_traceability | 产物→源版本映射（SBOM 雏形） | lint | SWE-085、SWEBOK CM |

### adds_auth_surface（鉴权/权限）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| threat_model | 攻击面登记并有评审签名 | manual + lint | SWE-154-159（安全） |
| least_privilege | 权限边界最小化 | test | SWE-155 |

### adds_observability（日志/指标/追踪）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| failure_evidence | 故障可由结构化证据定位 | test | 本仓库先例 O-022（cdb） |
| metric_anchoring | 运行时指标与 G 账预算联动 | lint（G.verification 引用） | SWE-090（测量） |

### goes_multi_instance（多实例/分布式部署）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| split_brain | 分区/重播/仲裁语义 | TLA+（协议层主场） | SWEBOK Architecture |
| instance_consistency | 实例间一致性级别声明 | test + model | SWE-052 |

## 视角覆盖标尺（Arcadia，G 账起草自检，非触发器）

每个目标起草时过五问：运营需要（谁在什么场景用它做什么）→ 系统行为
（它必须做什么/绝不做什​​么）→ 逻辑分解（由哪些部件协作）→ 物理承载
（跑在哪、怎么部署）→ 构建集成（怎么造出来、怎么验证造对了）。
五问中任何一问答不出可测性质，该目标标 draft 不许升 accepted。

## 明确不收编的域

管理/经济/职业伦理（SWEBOK Management/Economics/Professional）——
属人的裁决层，宪法硬/软二分已覆盖；机械检查它们是伪精确（SG6）。

---

## 候选触发器组（AI 时代域，现库零覆盖）

来源：**对若干成熟系统的定向代码阅读**（读规律，不记项目；实例按机制描述，
完整出处留在提出方的读数记录，核验状态见 `thinking/CANDIDATES.md`）。
与上列九组的差别：九组补的是 SWEBOK 的**弱域**，本组补的是**一种新的系统形状**——
"开放类型 + 外部裁决者 + 派生状态 + 追加日志"这一族在现库四个触发器里没有任何对应物，
于是下游项目实测到的规律**无法以义务形式入账**。

### adds_derived_state（引入缓存/派生态/捷径副本）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| recompute_equivalence | 派生值必须能由来源重算且相等；缓存与重算的比对要报出**首个差异位置**而非"不相等" | test（重算比对） | 外部实测（机制见 thinking/CANDIDATES.md §一） |
| independent_observation | **校验者不得与被校验者同体**——"调用同一变更操作来验证该操作"不构成不变量 | lint（在接口/不变量上强制登记独立观察源） | BS-012 + 外部独立收敛（同文件 §一 BS-012 行） |

### adds_external_decider（LLM 或其它外部神谕参与决策）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| envelope_bounds | 有界**语义**资源 + 具名回收策略（不是队列容量） | lint / test | 外部实测：信封四元素（机制见 thinking/CANDIDATES.md） |
| fail_closed_gate | 超限**拒绝**而非排队；`Defer/Reject` 是内核策略 | test | 同上 |
| decision_not_guaranteed | 裁决者**决策正确性不保证**，只保证其影响被信封围住——此句必须成文 | manual（签名） | 同上 |
| adjudicator_out_of_reach | 最终裁决者必须在被裁决者**够不着**处（判据不得由神谕自证） | lint（判据清单） | 外部实测 + 本仓先例思路 |

### adds_open_type_registry（数据/运行期可注册新类型）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| declaration_is_validator | 声明必须产生**真约束**（值域/枚举/引用存在/可否插值），不得只是编辑器提示 | property test | 外部实测：取值范围"提示"不产生任何运行期校验 |
| freeze_then_readonly | 注册—**全量**校验—冻结；冻结后声明不可变 | lint | 同上（反例：允许逐实例改写声明的回调） |
| empty_artifact_rejected | **空注册项拒收**，且"未发布"必须写明理由并被检查器读到 | lint | 外部实测：两百余个空伴生入口（同文件 §三） |

### adds_stable_identity（可寻址的元素/键/区间）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| no_index_addressing | 不得按位置寻址（插入/删除即失效，并逼出防御性缓存） | lint | 外部实测：按位置寻址的可编辑集合 |
| identity_survives_edit | 编辑后 id 仍指向同一对象；地址派生的身份禁止入库 | test | 同上（其原作者自认该做法为权宜） |

### adds_alternate_representation（引入压缩/紧凑/第二表示）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| no_representation_leak | 内部表示不得上升为用户可见概念或编辑态约束；紧凑格式只能是构建期一次性 lowering | lint（表示类型不得出现在 UI/编辑模块） | 外部实测：第二表示导致数十处分支 + 编辑器约十处交互守卫 |

### adds_append_only_log（日志成为唯一真相）
| 义务键 | 内容 | 建议检查器 | 来源 |
|---|---|---|---|
| single_fold | 只有一个重放/折叠函数；第二折叠器必须被结构堵死 | lint | 外部实测：一系统一条日志两个折叠器；另一系统用"未注册即抛"堵死 |
| refusals_logged | 拒绝也被记录（下一轮读的是状态，不是返回值） | test | 外部实测：拒绝做成普通结果入日志 |
| unknown_event_refuses_rebuild | 不认识的记录必须**拒绝重建**，不得静默跳过 | test | 外部实测：未知记录标记缺席即"必需" |
