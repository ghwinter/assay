//! 证据生成器：O-013(不变量) O-014(保序) O-015(吞吐, #[ignore]) O-016/017(唤醒/关闭) O-018(背压)
//! 运行: cargo test            （全部断言测试）
//!       cargo test --release -- --ignored --nocapture bench_throughput   （吞吐基准）
use bounded_queue::BoundedQueue;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ---------- O-017 + O-013 基础语义 ----------
#[test]
fn basic_close() {
    let q = BoundedQueue::new(4);
    assert!(q.try_pop().is_none()); // 空队列
    for i in 0..4 {
        assert!(q.push(i));
    }
    assert_eq!(q.size(), 4); // O-013: size <= cap
    q.close();
    assert!(!q.push(99)); // 关闭后 push 拒绝
    for i in 0..4 {
        assert_eq!(q.pop(), Some(i)); // 排空仍按序交付
    }
    assert_eq!(q.pop(), None); // 排空 + 关闭 → None
}

// ---------- O-017: 阻塞中的 popper 被 close 唤醒 ----------
#[test]
fn close_wakes_blocked_poppers() {
    let q = Arc::new(BoundedQueue::<i32>::new(2));
    let got = Arc::new(AtomicI32::new(0));
    let mut ts = vec![];
    for _ in 0..3 {
        let q = q.clone();
        let got = got.clone();
        ts.push(thread::spawn(move || {
            if q.pop().is_some() {
                got.fetch_add(1, Ordering::SeqCst); // 阻塞在空队列上
            }
        }));
    }
    thread::sleep(Duration::from_millis(100));
    q.close(); // 必须唤醒全部
    for t in ts {
        t.join().unwrap(); // 唤醒丢失 → 这里挂死
    }
    assert_eq!(got.load(Ordering::SeqCst), 0);
}

// ---------- O-018: 队满背压，消费后苏醒 ----------
#[test]
fn backpressure() {
    let q = Arc::new(BoundedQueue::new(1));
    assert!(q.push(1)); // 满
    let pushed = Arc::new(AtomicBool::new(false));
    let p_q = q.clone();
    let p_pushed = pushed.clone();
    let producer = thread::spawn(move || {
        p_pushed.store(p_q.push(2), Ordering::SeqCst); // 应阻塞
    });
    thread::sleep(Duration::from_millis(100));
    assert!(!pushed.load(Ordering::SeqCst)); // 仍阻塞 → 背压生效
    assert_eq!(q.pop(), Some(1));
    producer.join().unwrap();
    assert!(pushed.load(Ordering::SeqCst));
}

// ---------- O-013: MPMC 压力——恰好一次 + 逐操作不变量 ----------
#[test]
fn stress_mpmc_exactly_once() {
    const P: i64 = 4;
    const C: usize = 4;
    const N: i64 = 5000;
    const CAP: usize = 8;
    let q = Arc::new(BoundedQueue::<i64>::new(CAP));
    let broken = Arc::new(AtomicBool::new(false));
    let received: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::with_capacity((P * N) as usize)));

    let mut cs = vec![];
    for _ in 0..C {
        let q = q.clone();
        let received = received.clone();
        let broken = broken.clone();
        cs.push(thread::spawn(move || {
            while let Some(v) = q.pop() {
                if q.size() > CAP {
                    broken.store(true, Ordering::SeqCst); // O-013
                }
                received.lock().unwrap().push(v);
            }
        }));
    }
    let mut ps = vec![];
    for p in 0..P {
        let q = q.clone();
        let broken = broken.clone();
        ps.push(thread::spawn(move || {
            // p 按值进入 move 闭包：线程生存期长于循环变量
            for i in 0..N {
                if !q.push(p * N + i) {
                    broken.store(true, Ordering::SeqCst);
                    return;
                }
                if q.size() > CAP {
                    broken.store(true, Ordering::SeqCst); // O-013
                }
            }
        }));
    }
    for t in ps {
        t.join().unwrap();
    }
    q.close();
    for t in cs {
        t.join().unwrap();
    }

    assert!(!broken.load(Ordering::SeqCst));
    let mut got = received.lock().unwrap().clone();
    got.sort_unstable();
    let want: Vec<i64> = (0..P * N).collect();
    assert_eq!(got, want); // 恰好一次（不丢不重）
}

// ---------- O-014: 交付保序——单消费者下逐生产者子序列递增 ----------
// 多消费者下接收记录发生在队列锁之外，相邻交付的记录顺序无法从外部区分；
// 单消费者时接收序 == 交付序，保序可精确断言（与 C++ 归档实现 O-006 同一可检验形式）。
#[test]
fn fifo_single_consumer() {
    const P: usize = 4;
    const N: i64 = 5000;
    const CAP: usize = 8;
    let q = Arc::new(BoundedQueue::<i64>::new(CAP));
    let broken = Arc::new(AtomicBool::new(false));
    let order: Arc<Mutex<Vec<i64>>> = Arc::new(Mutex::new(Vec::with_capacity(P * N as usize)));

    let c_q = q.clone();
    let c_order = order.clone();
    let consumer = thread::spawn(move || {
        while let Some(v) = c_q.pop() {
            c_order.lock().unwrap().push(v); // 唯一消费者写，join 后主线程读
        }
    });
    let mut ps = vec![];
    for p in 0..P {
        let q = q.clone();
        let broken = broken.clone();
        ps.push(thread::spawn(move || {
            for i in 0..N {
                if !q.push((p as i64) * N + i) {
                    broken.store(true, Ordering::SeqCst);
                    return;
                }
            }
        }));
    }
    for t in ps {
        t.join().unwrap();
    }
    q.close();
    consumer.join().unwrap();

    assert!(!broken.load(Ordering::SeqCst));
    let order = order.lock().unwrap();
    assert_eq!(order.len(), P * N as usize);
    let mut next = [0i64; P];
    for &v in order.iter() {
        let p = (v / N) as usize;
        assert_eq!(v, (p as i64) * N + next[p]); // 该生产者的下一个元素必须正好是它
        next[p] += 1;
    }
}

// ---------- O-024: p99 阻塞延迟预算（G-003 v1，实验 B 注入） ----------
// 场景定义（预算的一部分）：4 生产者 / 4 消费者、容量 4；消费者每次 pop 后自旋 2µs
// 制造竞争；度量每个 push 的端到端延迟（含满队等待与 condvar 唤醒），p99 < 10µs。
#[test]
#[ignore]
fn bench_p99() {
    const P: usize = 4;
    const C: usize = 4;
    const N: u64 = 25000;
    const CAP: usize = 4;
    const SPIN_NS: u128 = 2000; // 消费者自旋时长
    let q = Arc::new(BoundedQueue::<u64>::new(CAP));
    let lat: Arc<Mutex<Vec<u32>>> = Arc::new(Mutex::new(Vec::new()));

    let mut cs = vec![];
    for _ in 0..C {
        let q = q.clone();
        cs.push(thread::spawn(move || {
            while let Some(_v) = q.pop() {
                let t = Instant::now();
                while t.elapsed().as_nanos() < SPIN_NS {} // 自旋：模拟下游处理
            }
        }));
    }
    let mut ps = vec![];
    for _ in 0..P {
        let q = q.clone();
        let lat = lat.clone();
        ps.push(thread::spawn(move || {
            let mut local: Vec<u32> = Vec::with_capacity(N as usize);
            for i in 0..N {
                let t = Instant::now();
                q.push(i);
                local.push(t.elapsed().as_nanos() as u32);
            }
            lat.lock().unwrap().extend(local);
        }));
    }
    for t in ps {
        t.join().unwrap();
    }
    q.close();
    for t in cs {
        t.join().unwrap();
    }
    let mut all = lat.lock().unwrap().clone();
    all.sort_unstable();
    let n = all.len();
    let p50 = all[n / 2];
    let p99 = all[n * 99 / 100];
    println!(
        "bench_p99 {P}x{C} x{N}: {} pushes, p50={p50}ns, p99={p99}ns",
        n
    );
    const BUDGET_NS: u32 = 10_000; // G-003 v1 预算——已由 D-013 方案A 撤回（synthetic）
    println!("  G-003 v1 预算 {BUDGET_NS}ns: 已撤回（D-013 方案A），本基准转报告型仪表，不判废");
}

// ---------- O-015: 吞吐基准，对照 G-003 预算 ----------
#[test]
#[ignore]
fn bench_throughput() {
    const P: usize = 4;
    const C: usize = 4;
    const N: i64 = 50000;
    let q = Arc::new(BoundedQueue::<i64>::new(1024));
    let total = Arc::new(AtomicI64::new(0));

    let mut cs = vec![];
    for _ in 0..C {
        let q = q.clone();
        let total = total.clone();
        cs.push(thread::spawn(move || {
            let mut n: i64 = 0;
            while q.pop().is_some() {
                n += 1;
            }
            total.fetch_add(n, Ordering::SeqCst);
        }));
    }
    let mut ps = vec![];
    let t0 = Instant::now();
    for _ in 0..P {
        let q = q.clone();
        ps.push(thread::spawn(move || {
            for i in 0..N {
                q.push(i);
            }
        }));
    }
    for t in ps {
        t.join().unwrap();
    }
    q.close();
    for t in cs {
        t.join().unwrap();
    }
    let us = t0.elapsed().as_micros() as f64;
    let ops = total.load(Ordering::SeqCst) as f64 * 1e6 / us;
    println!(
        "bench {P}x{C} x{N}: {} ops, {:.0} ops/s",
        total.load(Ordering::SeqCst),
        ops
    );
    const BUDGET: f64 = 50_000.0; // G-003 v0 预算
    assert!(
        ops >= BUDGET,
        "低于 G-003 预算 {BUDGET} ops/s（实测 {ops:.0}）"
    );
    println!("  G-003 预算 {BUDGET} ops/s: ok");
}

// ---------- O-030: timed_pop 谓词循环唤醒正确性（D-019） ----------
#[test]
fn timed_pop_timeout_returns_none() {
    let q = BoundedQueue::<i32>::new(2);
    let t0 = Instant::now();
    assert_eq!(q.timed_pop(Duration::from_millis(200)), None); // 空且未关闭 → 超时
    let ms = t0.elapsed().as_millis();
    assert!((150..=2000).contains(&ms), "超时应接近 200ms，实测 {ms}ms");
    assert_eq!(q.size(), 0); // 超时路径不动队列状态（O-030）
                             // 超时后队列仍可正常使用
    assert!(q.push(1));
    assert_eq!(q.pop(), Some(1));
}

#[test]
fn timed_pop_delivers_midwait() {
    let q = Arc::new(BoundedQueue::<i32>::new(2));
    let wq = q.clone();
    let t = thread::spawn(move || wq.timed_pop(Duration::from_secs(5)));
    thread::sleep(Duration::from_millis(100)); // 让消费者先阻塞
    assert!(q.push(42));
    assert_eq!(t.join().unwrap(), Some(42)); // 等待中交付即时返回，不耗尽 5s
}

#[test]
fn timed_pop_fifo_unchanged() {
    let q = BoundedQueue::<i32>::new(4);
    for i in 0..4 {
        assert!(q.push(i));
    }
    for i in 0..4 {
        assert_eq!(q.timed_pop(Duration::from_millis(50)), Some(i)); // 非阻塞路径序不变
    }
    assert_eq!(q.timed_pop(Duration::from_millis(50)), None);
}

// ---------- O-031: timed_pop 的关闭语义（D-019） ----------
#[test]
fn timed_pop_close_wakes_immediately() {
    let q = Arc::new(BoundedQueue::<i32>::new(2));
    let wq = q.clone();
    let t = thread::spawn(move || wq.timed_pop(Duration::from_secs(10)));
    thread::sleep(Duration::from_millis(100)); // 让消费者阻塞在 wait_timeout
    let t0 = Instant::now();
    q.close(); // 必须立即唤醒，不能等 10s（O-031）
    assert_eq!(t.join().unwrap(), None);
    assert!(t0.elapsed() < Duration::from_secs(5), "close 后应立即唤醒");
}

#[test]
fn timed_pop_closed_empty_immediate() {
    let q = BoundedQueue::<i32>::new(2);
    q.close();
    let t0 = Instant::now();
    assert_eq!(q.timed_pop(Duration::from_secs(10)), None); // 关闭且排空 → 不等待
    assert!(
        t0.elapsed() < Duration::from_millis(500),
        "关闭后不应消耗 timeout"
    );
}
