---- MODULE queue ----
(***************************************************************************)
(* BoundedQueue 协议模型（对应 pilot-rs 主实现的并发协议）                   *)
(* 决策来源：D-006 / D-009；检查器：TLC 模型检查                             *)
(*                                                                           *)
(* 建模约定：                                                                *)
(*   - 生产者 p 有 ITEMS 条待发消息（toSend[p]，按序），buf 为队列内容        *)
(*   - 阻塞显式建模为 pc 状态（waitfull / waitempty），而非 stuttering        *)
(*   - close 即 D-009/O-017 语义：唤醒全部阻塞者，排空后拒绝                  *)
(* 安全性质：TypeOK / CapInv / Partition(恰好一次+不丢) / DoneImpliesClosed   *)
(* 活性性质：close 后必然终止；阻塞的消费者/生产者在 close 后必然被唤醒        *)
(***************************************************************************)
EXTENDS Naturals, Sequences, Bags, FiniteSets

CONSTANTS PROD, CONS, CAP, ITEMS

Items(p) == {p * 10 + i : i \in 1..ITEMS}
AllItems == UNION {Items(p) : p \in 1..PROD}

(* 消息 e 在序列 s 中出现次数；序列内无重复判定 *)
Occurs(s, e) == Cardinality({j \in 1..Len(s) : s[j] = e})
NoDup(s) == \A i \in 1..Len(s): \A j \in 1..Len(s): s[i] = s[j] => i = j

PCP == {"work", "waitfull", "done"}
PCC == {"work", "waitempty", "done"}

VARIABLES buf, closed, pcP, pcC, toSend, recvd
vars == <<buf, closed, pcP, pcC, toSend, recvd>>

Init ==
  /\ buf = <<>>
  /\ closed = FALSE
  /\ pcP = [p \in 1..PROD |-> "work"]
  /\ pcC = [c \in 1..CONS |-> "work"]
  /\ toSend = [p \in 1..PROD |-> [i \in 1..ITEMS |-> p*10 + i]]
  /\ recvd = <<>>

(***************************)
(* 生产者动作               *)
(***************************)
PSend(p) ==
  /\ pcP[p] = "work"
  /\ toSend[p] # <<>>
  /\ ~closed
  /\ Len(buf) < CAP
  /\ buf' = Append(buf, toSend[p][1])
  /\ toSend' = [toSend EXCEPT ![p] = Tail(toSend[p])]
  /\ UNCHANGED <<closed, pcP, pcC, recvd>>

PClosed(p) ==   (* push 返回 false：关闭即拒绝（O-017） *)
  /\ pcP[p] = "work"
  /\ closed
  /\ toSend[p] # <<>>
  /\ pcP' = [pcP EXCEPT ![p] = "done"]
  /\ UNCHANGED <<buf, closed, pcC, toSend, recvd>>

PDone(p) ==
  /\ pcP[p] = "work"
  /\ toSend[p] = <<>>
  /\ pcP' = [pcP EXCEPT ![p] = "done"]
  /\ UNCHANGED <<buf, closed, pcC, toSend, recvd>>

PBlock(p) ==
  /\ pcP[p] = "work"
  /\ ~closed
  /\ toSend[p] # <<>>
  /\ Len(buf) = CAP
  /\ pcP' = [pcP EXCEPT ![p] = "waitfull"]
  /\ UNCHANGED <<buf, closed, pcC, toSend, recvd>>

PWake(p) ==     (* 容量释放或关闭后苏醒（O-016） *)
  /\ pcP[p] = "waitfull"
  /\ (Len(buf) < CAP \/ closed)
  /\ pcP' = [pcP EXCEPT ![p] = "work"]
  /\ UNCHANGED <<buf, closed, pcC, toSend, recvd>>

(***************************)
(* 消费者动作               *)
(***************************)
CPop(c) ==
  /\ pcC[c] = "work"
  /\ buf # <<>>
  /\ buf' = Tail(buf)
  /\ recvd' = Append(recvd, Head(buf))
  /\ UNCHANGED <<closed, pcP, pcC, toSend>>

CBlock(c) ==
  /\ pcC[c] = "work"
  /\ buf = <<>>
  /\ ~closed
  /\ pcC' = [pcC EXCEPT ![c] = "waitempty"]
  /\ UNCHANGED <<buf, closed, pcP, toSend, recvd>>

CWake(c) ==     (* 有元素或关闭后苏醒（O-016） *)
  /\ pcC[c] = "waitempty"
  /\ (buf # <<>> \/ closed)
  /\ pcC' = [pcC EXCEPT ![c] = "work"]
  /\ UNCHANGED <<buf, closed, pcP, toSend, recvd>>

CExit(c) ==     (* 排空且关闭后 pop 返回 None（O-017） *)
  /\ pcC[c] = "work"
  /\ buf = <<>>
  /\ closed
  /\ pcC' = [pcC EXCEPT ![c] = "done"]
  /\ UNCHANGED <<buf, closed, pcP, toSend, recvd>>

(***************************)
(* 关闭与终态               *)
(***************************)
Close ==
  /\ closed = FALSE
  /\ closed' = TRUE
  /\ UNCHANGED <<buf, pcP, pcC, toSend, recvd>>

Terminated ==
  /\ \A p \in 1..PROD: pcP[p] = "done"
  /\ \A c \in 1..CONS: pcC[c] = "done"

Tick == Terminated /\ UNCHANGED vars   (* 终态自环：保持 TLC 死锁检查对非终态有效 *)

PActs(p) == \/ PSend(p) \/ PClosed(p) \/ PDone(p) \/ PBlock(p) \/ PWake(p)
CActs(c) == \/ CPop(c) \/ CBlock(c) \/ CWake(c) \/ CExit(c)

Next ==
  \/ \E p \in 1..PROD: PActs(p)
  \/ \E c \in 1..CONS: CActs(c)
  \/ Close
  \/ Tick

(***************************)
(* 安全不变量               *)
(***************************)
TypeOK ==
  /\ buf \in Seq(AllItems)
  /\ closed \in BOOLEAN
  /\ pcP \in [1..PROD -> PCP]
  /\ pcC \in [1..CONS -> PCC]
  /\ toSend \in [1..PROD -> Seq(AllItems)]
  /\ recvd \in Seq(AllItems)

CapInv == Len(buf) <= CAP

(* 恰好一次 + 不丢：每条消息在 buf、recvd、各 toSend 中恰好出现一次 *)
Partition ==
  /\ \A p \in 1..PROD: NoDup(toSend[p])
  /\ \A e \in AllItems:
       Occurs(buf, e) + Occurs(recvd, e)
         + Cardinality({p \in 1..PROD : Occurs(toSend[p], e) > 0}) = 1

(* 生产者终止但仍有未发消息 ⇒ 一定是因为关闭（否则即悄悄丢消息） *)
DoneImpliesClosed ==
  \A p \in 1..PROD: (pcP[p] = "done" /\ toSend[p] # <<>>) => closed

(***************************)
(* 公平性与活性             *)
(***************************)
Fairness ==
  /\ \A p \in 1..PROD: /\ WF_vars(PSend(p))
                       /\ WF_vars(PClosed(p))
                       /\ WF_vars(PDone(p))
                       /\ WF_vars(PWake(p))
  /\ \A c \in 1..CONS: /\ WF_vars(CPop(c))
                       /\ WF_vars(CWake(c))
                       /\ WF_vars(CExit(c))

Spec == Init /\ [][Next]_vars /\ Fairness

(* 活性主性质：close 之后所有线程必然终止。
   它蕴含逐线程的"无丢失唤醒"——任一阻塞者若不被唤醒，Terminated 即不可达。
   逐线程的 \A-LeadsTo 形式 TLC 无法检查（不支持量化时序公式），故以本性质为检。
   注意：时序公式直接内联——TLC 不支持经参数化算子包装的时序体。 *)
AfterCloseTerminate == [](closed => <>(Terminated))

====
