# Terminology Control — 《压缩与验证》/ *Compression and Verification*

**Role of this file.** The English build (`en/`) is the authoritative text. The Chinese
build (`zh/`) is a controlled rendering of it. This file fixes the one-to-one mapping
between the two, and records every place where a Chinese word would otherwise be
polysemous and could bias the reading.

**Rule.** A technical term is written in **exactly one way** in each language. Where a
natural Chinese word covers two distinct English concepts, the Chinese is split and the
split is recorded below.

---

## 1. Load-bearing terms

| English (authoritative) | Chinese | Definition / scope | Ambiguity resolved |
|---|---|---|---|
| **level** | 层级 | The formal object Λ = (X, 𝓑, α, A, C, 𝓘); a *cognitive scale*, defined relative to a study object and a task | In Chinese, 层 alone also means "layer" and "tier". An implementation tier is **layer** = 层; a stage of the refinement chain is **stage** = 阶段. Only 层级 maps to *level*. |
| **layer** | 层 | An implementation tier: formal layer / intent layer / execution layer (§7) | Never used for Λ. |
| **stage** | 阶段 | One of G1–G7 in the refinement chain | Never used for Λ. |
| **oblivious set** | 无视集 | ker α, the indistinguishability relation of a level | Not "ignored set", not "neglected set": the level *cannot* see the difference, it does not choose to disregard it. |
| **abstraction map** | 抽象映射 | α : 𝓑_{i+1} → 𝓑_i | 抽象 as a bare noun is avoided; the map is always 抽象映射 and the act is 抽象化. |
| **concretisation map** | 具体化映射 | γ : 𝓑_i → 2^{𝓑_{i+1}} | — |
| **compression ratio** | 压缩率 | κ = information annihilated by α | Not a data-compression ratio in the Shannon sense; it is measured *relative to a task*. |
| **structural convergence** | 结构收敛 | The phenomenon of §3 | 收敛 alone is reserved for this. Iterative convergence (of an estimator) is 收敛性 and never appears bare. |
| **domain binding** | 领域绑定度 | Dom(Λ), fraction of domain-specific axioms | — |
| **internal infrastructuralisation** | 内部基础设施化 | The three-phase process of Def. 4.1 | Not "infrastructure build-out". |
| **the six functions** | 内核六职能 | 𝓚₆ | 职能, not 功能: *function* as an office/duty, deliberately distinct from 函数 (mathematical function). |
| **kernel** | 内核 | As in OS kernel; the substrate that defines participant and arbitration | — |
| **substrate** | 基座 | The unified mechanism set 𝓤 | Deliberately not 基础设施, which is reserved for *infrastructure* as a general noun. |
| **participant** | 参与者 | An element of E | Not "actor": 行动者 is avoided to prevent confusion with the actor model. |
| **integration cost** | 集成成本 | Theorem 5.1 | — |
| **phase transition** | 相变 | Crossing of the threshold | Kept from physics; it is a genuine change of scaling regime. |
| **read-set** | 读集 | ρ(c) | Not "reading load". |
| **enforcement site** | 强制点 | S(ι) | — |
| **invariant** | 不变量 | ι ∈ 𝓘 | — |
| **circumvent** | 绕过 | An enforcement site being bypassed | Not 规避, which suggests avoidance rather than violation. |
| **near-decomposability** | 近可分解性 | Simon's term, here given a sufficient-statistic criterion | — |
| **sufficient statistic** | 充分统计量 | Standard statistics | — |
| **leak pair** | 泄漏对 | Two executions with equal interface trace and different external behaviour | Not "leakage": 泄漏 here is *abstraction leakage*, unrelated to information leakage (信息泄漏), which is a security term. This collision is flagged wherever both could occur. |
| **technical debt** | 技术债 | ℓ(code) − ℓ*(code) | — |
| **rate–distortion** | 率失真 | Shannon's theory | — |
| **governable complexity** | 可治理复杂度 | C(t) = E[ρ]·λ | 治理 (govern) is chosen over 管理 (manage) to keep it distinct from resource management in 𝓚₆. |
| **comprehension capacity** | 理解容量 | K = h·b | — |
| **scaling exponent** | 标度指数 | α in h ∝ n^{1+α} | 标度 = scaling in the physics sense; 规模 = size; 尺度 = scale (of separation). Three distinct Chinese words for three distinct English words. |
| **scale separation** | 尺度分离 | The property nature gets free and software must manufacture | — |

## 2. Assurance vocabulary (the most dangerous area)

| English | Chinese | Meaning | Explicitly **not** |
|---|---|---|---|
| **verification** | 验证 | Does the implementation meet the specification? | Not validation. |
| **validation** | 确认 | Does the system meet the real need? | Not verification. |
| **assurance** | 保证 | The conditioned relation of Def. 2.11 | — |
| **guarantee** | 保证 | An individual commitment by a level or component | Chinese does not distinguish these; where the distinction is load-bearing the English gloss is printed in parentheses. |
| **attestation / approval** | 审定 | A human or consensus act of adopting a specification | **Deliberately split from 确认.** Earlier drafts used 确认 for both validation and human approval; the collision made (g3) of Prop. 7.2 unreadable. |
| **ascertain (that)** | 核实 | To check that a deployed artefact is the verified one | Split from 确认 for the same reason. |
| **evidence** | 证据 | Test results, certificates, traces | Not proof. |
| **proof** | 证明 | A machine-checkable proof object | Not evidence. |
| **decide / decidable** | 判定 / 可判定 | Decidability in the computability sense | 决定 is reserved for "determine" in the causal sense. |
| **admission** | 准入 | The gate that lets an artefact run | — |
| **rigid / deterministic** | 确定 | The property of the admission gate | **Changed from 死板**, which is colloquial. |
| **untrusted** | 不可信 | Of a generator | — |
| **artefact** | 制品 | A build output | Not 工件, which suggests a machined part. |
| **artefact binding** | 制品绑定 | Eq. (2.11) | The "fourth pillar" = 第四支柱. |
| **trusted computing base** | 可信计算基 | TCB | — |

## 3. Specification vocabulary

| English | Chinese | Note |
|---|---|---|
| **specification** | 规格 | The formal object Φ. **All** occurrences; earlier drafts mixed 规范 in. |
| **normative** | 规范性的 | The adjective only. 规范 as a bare noun is no longer used for the formal object. |
| **requirement** | 需求 | Pre-formal. |
| **intent** | 意图 | The human-side object I. |
| **convention** | 约定 / 工作规范 | Working practice, not a formal object. |
| **property** | 性质 | **Unified**; 属性 has been removed (2 occurrences). |
| **feature** | 特性 | A countable capability of the product; distinct from *property*. Kept, and never used for 性质. |
| **constraint** | 约束 | — |
| **contract** | 契约 | — |
| **contract operad** | 契约操作元 | — |
| **colour** | 颜色 | Operad colouring. |

## 4. Notation and its reading

| Symbol | English reading | Chinese reading |
|---|---|---|
| Λ | level | 层级 |
| 𝓑 | behaviour set | 行为集合 |
| α, γ | abstraction, concretisation | 抽象映射、具体化映射 |
| Obl(Λ) = ker α | oblivious set | 无视集 |
| κ | compression ratio | 压缩率 |
| 𝒢 | architecture relational graph | 架构关系型 |
| φ | skeleton map | 骨架映射 |
| T = (Q, d) | task = (query family, loss) | 任务 |
| ℓ(M) | description length | 描述长度 |
| L_T | task loss | 任务损失 |
| S = (P,Q,E⁺,E⁻,R,I,F,A) | behaviour contract | 行为契约 |
| Tr(B,E) ⊆ Allowed(Φ) | trace inclusion | 轨迹包含 |
| 𝔘 | assurance | 保证 |
| 𝓚₆ | the six functions | 内核六职能 |
| ρ(c) | read-set | 读集 |
| C(t), K | governable complexity, comprehension capacity | 可治理复杂度、理解容量 |
| 𝒥, rank_𝒯 𝒥 | observation operator, observability rank | 观测算子、观测秩 |
| q | correlated error mode | 相关错误模式 |

## 5. Changes applied to the Chinese in this revision

1. **确认 → 审定** (13 occurrences) where the sense was *human/consensus approval*.
   `确认` now occurs only in the *validation* sense (6 occurrences).
2. **确认 → 核实** (1) for "ascertain that the running binary is the verified one".
3. **担保 → 保证** (5) — the split carried no information.
4. **死板 → 确定** (4) — colloquial register.
5. **规范 → 规格** (4) where the formal object was meant; `规范` retained only as the
   adjective 规范性的.
6. **属性 → 性质** (2); **规约 → 规格/规范** (2).
7. Result: each English technical term above maps to exactly one Chinese term.

## 6. Terms deliberately *not* used, and why

| Rejected | Reason |
|---|---|
| 死板验证器 | colloquial; replaced by 确定性验证器 |
| 机械武器 (for constraint mechanisms) | militarised metaphor; replaced by 约束机制 |
| 越位 (for over-reaching arguments) | football metaphor; replaced by 被移出承重结构的论证 |
| 武器, 武器库 | same |
| 全知, 博士层级, 顶级专家 | self-description of the text's own standing; removed entirely |

## 7. Terms added in the first revision round

The revisions introduced new concepts, and several of them would otherwise have been rendered
into a Chinese word that is already bound to a *different* English term. Each such collision is
resolved below; the rest are recorded so that one English term keeps exactly one Chinese form.

### 7.1 Splits, where one Chinese word would have covered two English terms

| English (authoritative) | Chinese | Distinct from |
|---|---|---|
| **break-even threshold** | 盈亏平衡阈值／盈亏平衡点 | **phase transition** = 相变. The two are now different claims: crossing a break-even makes one arrangement cheaper, while a phase transition is a structural change of regime. Only 相变 may be used where the English genuinely says *phase transition*. |
| **identifiability** | 可辨识性 | **observability rank** = 观测秩. The rank condition is now the finite-dimensional *specialisation* of identifiability, not its general form. |
| **authorisation**, **authorised** | 授权／被授权 | **authority** = 权威; **standing** = 资格／采用资格. Authorisation is the act, authority the standing, and 权威同构 renders *authority isomorphism*. |
| **attestor** | 审定者 | **verifier** = 验证者. The English switches deliberately between the two where the mechanism changes; the Chinese must switch with it. |
| **re-confirmation** (of a specification) | 重新审定 | **re-verification** (of an artefact) = 重新验证. |
| **make-or-buy** | 自建／外购 | **adopt** (from a host) = 外借. |
| **stratum** | 层（stratum） | **layer** = 层. The English gloss is kept in the Chinese so the geometric sense cannot be read as the paper's *layer*. |
| **amortisation** | 摊销 | **amortised resource analysis** (the technique name) = 摊还. The two share a root but are different objects; the split is deliberate and predates this revision. |

### 7.2 New terms with a single binding

| English | Chinese | English | Chinese |
|---|---|---|---|
| unattested part | 未审定部分 | validity horizon | 有效视界 |
| checklist obligation | 清单式义务 | re-confirmation triggers | 重新审定触发条件 |
| deontic | 道义（道义轴） | design effect | 设计效应 |
| non-interference | 非干扰 | covert channel | 隐蔽信道 |
| logical depth | 逻辑深度 | bit-map encoding | 位图编码 |
| differential entropy | 微分熵 | coding theorem | 编码定理 |
| relational composition premise | 复合相容性包含 | boundary fidelity | 边界忠实性 |
| safety / liveness | 安全性／活性 | progress property | 进展性质 |
| well-foundedness | 良基性 | failure domain | 失效域 |
| silent failure | 静默失败 | governance cost | 治理成本 |
| switching cost | 转换成本 | zero set | 零点集 |
| regular value | 正则值 | general position | 一般位置 |
| compactness | 紧致性 | Pareto frontier | 帕累托前沿 |
| ablation | 消融实验 | finite differences | 有限差分 |
| head-of-line blocking | 队头阻塞 | taint tracking | 污点追踪 |
| channel auditing | 信道审计 | timing channel | 时间通道 |
| tail latency | 尾延迟 | functional property | 功能性性质 |
| agentic generation | 智能体式生成 | interpretation bands | 解释区间 |
| stipulated decision bands | 约定的判定区间 | liability regime | 法律责任制度 |
| institutional fact | 制度事实 | adjudicator | 裁决者 |
| multi-objective | 多目标 | resource algebra | 资源代数 |
| irredundant | 冗余自由的 | rework | 返工 |

### 7.3 How this section is maintained

A term enters this section when it is first used in either build. If a later revision changes the
English term, the Chinese changes with it in the same commit; the two builds are not allowed to
drift, and the label-and-reference parity check between `en/` and `zh/` is the mechanical guard
that catches drift in the structure even when the prose looks right.
