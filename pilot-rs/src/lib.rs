//! pilot-rs/src/lib.rs — BoundedQueue<T>：有界 MPMC 阻塞队列（主实现，Rust）
//!
//! 决策来源：D-007（主语言 Rust）D-009（Mutex + 双 Condvar，无 unsafe）D-010（值语义与固定容量）
//! D-017（队列核心抽取为 qcore.rs 单一事实源，Verus 契约住在实现文件里）
//! 义务台账：O-011..O-020、O-028，见 ledger/decisions/；C++ 对照归档见 pilot/（D-008）
//!
//! 语义约定（与 C++ 归档实现一致）：
//!   push(v)  阻塞；队列关闭时返回 false（不会部分入队）
//!   pop()    阻塞；交付顺序 = 插入顺序（总 FIFO）；关闭且排空后返回 None
//!   close()  唤醒全部阻塞者；已入队元素仍可取，之后 push 一律拒绝
//!
//! 分层（检查器阶梯，D-017）：
//!   qcore.rs    —— 纯数据流核心：不变量/前后置条件由 Verus 机器证明（O-028）
//!   本文件壳层  —— Mutex/Condvar 阻塞时序：由 TLA+ 档协议模型背书（O-016/O-017），
//!                  对应关系机械登记于 ledger/bridge.yaml（O-025）

use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::qcore::QueueCore;

mod qcore;

pub struct BoundedQueue<T> {
    state: Mutex<State<T>>,
    not_empty: Condvar, // popper 等待
    not_full: Condvar,  // pusher 等待
    capacity: usize,
}

struct State<T> {
    core: QueueCore<T>,
    closed: bool,
}

impl<T> BoundedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        BoundedQueue {
            state: Mutex::new(State {
                core: qcore::qnew(capacity),
                closed: false,
            }),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn size(&self) -> usize {
        self.state.lock().expect("queue mutex poisoned").core.len()
    }

    /// 阻塞 push。返回 false 仅当队列已关闭。
    pub fn push(&self, value: T) -> bool {
        let mut st = self.state.lock().expect("queue mutex poisoned");
        while !st.closed && st.core.len() >= self.capacity {
            // O-016：谓词循环，虚假唤醒安全
            st = self.not_full.wait(st).expect("queue mutex poisoned");
        }
        if st.closed {
            return false; // O-017：关闭即拒绝
        }
        // owned 传递（Verus 契约形态，O-028）：锁内取还，placeholder 无人观测
        let taken = std::mem::take(&mut st.core);
        let (back, ok) = qcore::push(taken, value);
        st.core = back;
        drop(st);
        self.not_empty.notify_one();
        ok
    }

    /// 阻塞 pop。返回 None 仅当已关闭且已排空。
    pub fn pop(&self) -> Option<T> {
        let mut st = self.state.lock().expect("queue mutex poisoned");
        while st.core.len() == 0 && !st.closed {
            st = self.not_empty.wait(st).expect("queue mutex poisoned");
        }
        let taken = std::mem::take(&mut st.core);
        let (v, back) = qcore::pop(taken);
        st.core = back;
        drop(st);
        self.not_full.notify_one();
        v
    }

    /// 非阻塞尝试取一个（关闭与否不影响：有元素即取）。
    pub fn try_pop(&self) -> Option<T> {
        let mut st = self.state.lock().expect("queue mutex poisoned");
        let taken = std::mem::take(&mut st.core);
        let (v, back) = qcore::pop(taken);
        st.core = back;
        v
    }

    /// 限时 pop：最多等待 `timeout`（D-019）。
    ///
    /// - `Some(v)` = 交付；`None` = 超时，或已关闭且排空（两种情形同形，D-019 取舍）
    /// - 谓词循环与 `pop()` 同构：`wait_timeout` 的返回值不是真值来源，
    ///   谓词重检才是——超时唤醒与虚假唤醒因此不可区分（O-030）
    /// - 超时返回时队列状态不变，调用方可继续 `pop()` / `timed_pop()`
    /// - 极端 `timeout`（> ~2.6 亿年）会因 Instant 溢出 panic——文档级边界
    pub fn timed_pop(&self, timeout: Duration) -> Option<T> {
        let deadline = Instant::now() + timeout;
        let mut st = self.state.lock().expect("queue mutex poisoned");
        while st.core.len() == 0 && !st.closed {
            let now = Instant::now();
            if now >= deadline {
                return None; // 超时：状态未动，锁随 st 释放
            }
            let (g, _res) = self
                .not_empty
                .wait_timeout(st, deadline - now)
                .expect("queue mutex poisoned");
            st = g; // O-030：超时/虚假唤醒/交付/关闭统一回到谓词重检
        }
        if st.core.len() == 0 {
            return None; // O-031：关闭且排空，立即返回（循环已被 closed 放行）
        }
        let taken = std::mem::take(&mut st.core);
        let (v, back) = qcore::pop(taken);
        st.core = back;
        drop(st);
        self.not_full.notify_one();
        v
    }

    /// 关闭：唤醒全部阻塞者（O-017）。幂等。
    pub fn close(&self) {
        let mut st = self.state.lock().expect("queue mutex poisoned");
        st.closed = true;
        drop(st);
        self.not_empty.notify_all();
        self.not_full.notify_all();
    }
}

// O-011（竞争自由）：全模块零 unsafe 块——数据竞争自由由 Rust 类型系统（Send/Sync）
// 语言强制；O-012（死锁自由）：全类仅 state 一把锁，每方法至多持锁一次，无嵌套加锁；
// O-019（所有权边界）：core 仅存在于 Mutex<State> 内部，API 面无任何借用逃逸——编译器强制。
