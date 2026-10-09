// pilot-rs/src/qcore.rs — 队列核心：无锁纯数据流（D-017 单一事实源）
//
// 同一文件、双工具链：
//   cargo build/test            —— Verus 契约全部 cfg 门控，纯净零依赖可执行代码
//   verus --cfg verus_verify    —— 契约生效，机器证明（O-028；入口 proofs/queue_core.rs）
//
// 机制：契约以 `#[cfg_attr(verus_verify, verus_spec(...))]` 附着，spec 模块以
// `#[cfg(verus_verify)]` 门控——普通 rustc 看到的文件与手写实现无异，
// Verus 看到的是带前后置条件的同一份逻辑。验证对象即编译对象。
//
// API 形态：owned 值传递（取/还 QueueCore）而非 &mut self——本版本 Verus
// （0.2026.09.27）下 &mut 的 old/final 在调用链上存在 SMT 实例化缺口（单个
// 事实可证、组合不可证）；owned 使每个调用点都是显式等式（D-006 备忘延续）。

#[cfg(verus_verify)]
use vstd::prelude::*;
#[cfg(verus_verify)]
use verus_builtin_macros::{verus, verus_spec};

#[cfg(verus_verify)]
mod vspecs {
    use super::QueueCore;
    use vstd::prelude::*;

    verus! {
        #[verifier::external_type_specification]
        pub struct ExQueueCore<T>(QueueCore<T>);

        /// 核心不变量（O-013 的机器证明形式）：0 <= len <= cap
        pub open spec fn inv<T>(q: &QueueCore<T>) -> bool {
            q.q@.len() <= q.cap
        }
    }
}

#[cfg(verus_verify)]
pub use vspecs::inv;

pub struct QueueCore<T> {
    pub cap: usize,
    pub q: Vec<T>,
}

impl<T> QueueCore<T> {
    /// 占位值（mem::take 用）：cap=0、空队列——inv 成立，锁内短暂存在无人观测
    pub fn placeholder() -> Self {
        QueueCore { cap: 0, q: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.q.len()
    }
}

impl<T> Default for QueueCore<T> {
    fn default() -> Self {
        Self::placeholder()
    }
}

#[cfg_attr(verus_verify, verus_spec(q =>
    ensures
        inv(&q),
        q.q@.len() == 0,
        q.cap == cap,
))]
pub fn qnew<T>(cap: usize) -> QueueCore<T> {
    QueueCore { cap, q: Vec::new() }
}

/// 队尾追加；已满则拒绝并保持不变量与容量
#[cfg_attr(verus_verify, verus_spec(res =>
    requires inv(&q),
    ensures
        inv(&res.0),
        res.0.cap == q.cap,
        res.1 <==> q.q@.len() < q.cap,
        res.1 ==> res.0.q@ == q.q@.push(v),
        res.1 ==> res.0.q@.len() == q.q@.len() + 1,
        !res.1 ==> res.0.q@ == q.q@,
        !res.1 ==> res.0.q@.len() == q.q@.len(),
))]
pub fn push<T>(mut q: QueueCore<T>, v: T) -> (QueueCore<T>, bool) {
    if q.q.len() < q.cap {
        q.q.push(v);
        (q, true)
    } else {
        (q, false)
    }
}

/// 取队头；空则 None（关闭与否是 shell 层语义，core 不管）
#[cfg_attr(verus_verify, verus_spec(res =>
    requires inv(&q),
    ensures
        inv(&res.1),
        res.1.cap == q.cap,
        q.q@.len() > 0 ==>
            res.0 == Some(q.q@[0]) && res.1.q@ == q.q@.remove(0)
            && res.1.q@.len() == q.q@.len() - 1,
        q.q@.len() == 0 ==>
            res.0.is_none() && res.1.q@ == q.q@
            && res.1.q@.len() == q.q@.len(),
))]
pub fn pop<T>(mut q: QueueCore<T>) -> (Option<T>, QueueCore<T>) {
    if q.q.len() > 0 {
        let hd = q.q.remove(0);
        (Some(hd), q)
    } else {
        (None, q)
    }
}

#[cfg(verus_verify)]
verus! {
fn main() {
    let q0 = qnew::<u64>(2);
    assert(q0.q@.len() == 0);
    let (q1, r1) = push(q0, 11);
    assert(r1);
    assert(q1.q@[0] == 11);
    let (q2, r2) = push(q1, 12);
    assert(r2);
    assert(q2.q@.len() == 2);
    let (q3, r3) = push(q2, 13);
    assert(!r3);                    // 满则拒绝（不变量上界的机器检验）
    assert(q3.q@[0] == 11 && q3.q@[1] == 12);
    let (a, q4) = pop(q3);
    assert(a == Some(11));          // FIFO：先入先出
    let (b, q5) = pop(q4);
    assert(b == Some(12));
    let (c, q6) = pop(q5);
    assert(c.is_none());            // 空返回 None
    assert(q6.q@.len() == 0);       // 不变量下界
}
}
