// pilot/tests/test_queue.cpp — 证据生成器
// 覆盖义务: O-005(不变量) O-006(总FIFO) O-009(关闭语义) O-010(背压) O-007(吞吐)
// 决策来源: D-003(单锁+双condvar) D-004(固定容量)
// 用法: test_queue test | bench [P] [C] [N]
#include "queue.hpp"

#include <atomic>
#include <cassert>
#include <chrono>
#include <cstdio>
#include <cstdlib>
#include <map>
#include <mutex>
#include <string>
#include <thread>
#include <vector>

using Clock = std::chrono::steady_clock;

// ---------- O-009 + O-005 基础语义 ----------
static void test_basic() {
    BoundedQueue<int> q(4);
    int v = 0;
    assert(!q.try_pop(v));                       // 空队列
    for (int i = 0; i < 4; ++i) assert(q.push(i));
    assert(q.size() == 4);                       // O-005: size <= cap
    q.close();
    assert(!q.push(99));                         // 关闭后 push 拒绝
    for (int i = 0; i < 4; ++i) {                // 排空仍按序交付
        assert(q.pop(v));
        assert(v == i);
    }
    assert(!q.pop(v));                           // 排空 + 关闭 → false
    std::puts("  basic/close: ok");
}

// ---------- O-009: 阻塞中的 popper 被 close 唤醒 ----------
static void test_close_wakes_blocked_poppers() {
    BoundedQueue<int> q(2);
    std::atomic<int> got{0};
    std::vector<std::thread> ts;
    for (int k = 0; k < 3; ++k)
        ts.emplace_back([&] {
            int v;
            if (q.pop(v)) ++got;                 // 阻塞在空队列上
        });
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    q.close();                                   // 必须唤醒全部
    for (auto& t : ts) t.join();                 // 唤醒丢失 → 这里挂死
    assert(got.load() == 0);
    std::puts("  close wakes blocked poppers: ok");
}

// ---------- O-010: 队满背压，消费后苏醒 ----------
static void test_backpressure() {
    BoundedQueue<int> q(1);
    int v = 0;
    assert(q.push(1));                           // 满
    std::atomic<bool> pushed{false};
    std::thread producer([&] { pushed = q.push(2); });  // 应阻塞
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
    assert(!pushed.load());                      // 仍阻塞 → 背压生效
    assert(q.pop(v) && v == 1);
    producer.join();
    assert(pushed.load());
    std::puts("  backpressure: ok");
}

// ---------- O-005 + O-006a: MPMC 压力——恰好一次 + 逐操作不变量 ----------
static void test_stress() {
    constexpr int P = 4, C = 4, N = 5000, CAP = 8;
    BoundedQueue<int> q(CAP);
    std::atomic<long long> recv_seq{0};          // 全局接收序
    std::map<int, long long> recv_at;            // value → 接收序
    std::mutex m_at;
    std::atomic<int> consumed{0};
    std::atomic<bool> broken{false};

    std::vector<std::thread> cs;
    for (int c = 0; c < C; ++c)
        cs.emplace_back([&] {
            int v;
            while (q.pop(v)) {
                if (q.size() > (std::size_t)CAP) broken = true;   // O-005
                long long s = recv_seq.fetch_add(1);
                std::lock_guard<std::mutex> lk(m_at);
                recv_at[v] = s;
                ++consumed;
            }
        });
    std::vector<std::thread> ps;
    for (int p = 0; p < P; ++p)
        ps.emplace_back([&, p] {                 // p 按值捕获：线程生存期长于循环变量
            for (int i = 0; i < N; ++i) {
                int item = p * N + i;            // 全局唯一值
                if (!q.push(item)) { broken = true; return; }
                if (q.size() > (std::size_t)CAP) broken = true;   // O-005
            }
        });
    for (auto& t : ps) t.join();
    q.close();
    for (auto& t : cs) t.join();

    assert(!broken.load());
    assert(consumed.load() == P * N);                              // 恰好一次
    assert(recv_at.size() == static_cast<std::size_t>(P) * N);
    std::printf("  stress %dx%d x%d: ok (%d consumed)\n", P, C, N, consumed.load());
}

// ---------- O-006b: 交付保序——单消费者下逐生产者子序列递增 ----------
// 说明：多消费者下，接收记录发生在队列锁之外，相邻交付的记录顺序可能交换，
// 因此"总 FIFO"无法从外部精确检验；单消费者时 接收序 == 交付序，保序可精确断言。
static void test_fifo_single_consumer() {
    constexpr int P = 4, N = 5000, CAP = 8;
    BoundedQueue<int> q(CAP);
    std::atomic<bool> broken{false};
    std::vector<int> order;                      // 唯一消费者写，join 后主线程读
    order.reserve(static_cast<std::size_t>(P) * N);

    std::thread consumer([&] {
        int v;
        while (q.pop(v)) order.push_back(v);
    });
    std::vector<std::thread> ps;
    for (int p = 0; p < P; ++p)
        ps.emplace_back([&, p] {
            for (int i = 0; i < N; ++i)
                if (!q.push(p * N + i)) { broken = true; return; }
        });
    for (auto& t : ps) t.join();
    q.close();
    consumer.join();

    assert(!broken.load());
    assert(order.size() == static_cast<std::size_t>(P) * N);
    std::vector<int> next(P, 0);
    for (int v : order) {
        int p = v / N;
        assert(v == p * N + next[p]);            // 该生产者的下一个元素必须正好是它
        ++next[p];
    }
    std::puts("  fifo single-consumer: ok");
}

// ---------- O-007: 吞吐基准，对照 G-003 预算 ----------
static int run_bench(int P, int C, int N) {
    BoundedQueue<int> q(1024);
    std::atomic<long long> total{0};
    std::vector<std::thread> cs;
    auto t0 = Clock::now();
    for (int c = 0; c < C; ++c)
        cs.emplace_back([&] {
            int v;
            long long n = 0;
            while (q.pop(v)) ++n;
            total += n;
        });
    std::vector<std::thread> ps;
    for (int p = 0; p < P; ++p)
        ps.emplace_back([&] {
            for (int i = 0; i < N; ++i) q.push(1);
        });
    for (auto& t : ps) t.join();
    q.close();
    for (auto& t : cs) t.join();
    auto us = std::chrono::duration_cast<std::chrono::microseconds>(Clock::now() - t0).count();
    double ops = static_cast<double>(total.load()) * 1e6 / static_cast<double>(us);
    std::printf("bench %dx%d x%d: %lld ops, %.0f ops/s\n", P, C, N, total.load(), ops);
    const double budget = 50000.0;               // G-003 v0 预算
    if (ops < budget) {
        std::printf("  FAIL: 低于 G-003 预算 %.0f ops/s\n", budget);
        return 1;
    }
    std::printf("  G-003 预算 %.0f ops/s: ok\n", budget);
    return 0;
}

int main(int argc, char** argv) {
    std::string mode = argc > 1 ? argv[1] : "test";
    if (mode == "test") {
        test_basic();
        test_close_wakes_blocked_poppers();
        test_backpressure();
        test_stress();
        test_fifo_single_consumer();
        std::puts("ALL TESTS PASSED");
        return 0;
    }
    if (mode == "bench") {
        int P = argc > 2 ? std::atoi(argv[2]) : 4;
        int C = argc > 3 ? std::atoi(argv[3]) : 4;
        int N = argc > 4 ? std::atoi(argv[4]) : 50000;
        return run_bench(P, C, N);
    }
    std::fprintf(stderr, "usage: %s test | bench [P] [C] [N]\n", argv[0]);
    return 2;
}
