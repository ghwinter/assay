// pilot/src/queue.hpp — BoundedQueue<T>：有界 MPMC 阻塞队列（header-only）
//
// 决策来源：D-002（值语义与 API）D-003（单 mutex + 双 condvar）D-004（固定容量）
// 义务台账：O-001..O-010，见 ledger/decisions/；证据见 ledger/evidence/
//
// 语义约定：
//   push(v)  阻塞；队列关闭时返回 false（不会部分入队）
//   pop(v)   阻塞；交付顺序 = 插入顺序（总 FIFO）；关闭且排空后返回 false
//   close()  唤醒全部阻塞者；已入队元素仍可取，之后 push 一律拒绝
#pragma once

#include <condition_variable>
#include <cstddef>
#include <deque>
#include <mutex>
#include <utility>

template <typename T>
class BoundedQueue {
public:
    explicit BoundedQueue(std::size_t capacity) : capacity_(capacity) {}

    // 阻塞 push。返回 false 仅当队列已关闭。
    bool push(T value) {
        std::unique_lock<std::mutex> lk(m_);
        not_full_.wait(lk, [&] { return closed_ || q_.size() < capacity_; });
        if (closed_) return false;                 // O-009：关闭即拒绝
        q_.push_back(std::move(value));
        lk.unlock();
        not_empty_.notify_one();
        return true;
    }

    // 阻塞 pop。返回 false 仅当已关闭且已排空。
    bool pop(T& out) {
        std::unique_lock<std::mutex> lk(m_);
        not_empty_.wait(lk, [&] { return closed_ || !q_.empty(); });
        if (q_.empty()) return false;              // closed_ && drained
        out = std::move(q_.front());
        q_.pop_front();
        lk.unlock();
        not_full_.notify_one();
        return true;
    }

    // 非阻塞尝试取一个。
    bool try_pop(T& out) {
        std::lock_guard<std::mutex> lk(m_);
        if (q_.empty()) return false;
        out = std::move(q_.front());
        q_.pop_front();
        return true;
    }

    // 关闭：唤醒全部阻塞者（O-009）。幂等。
    void close() {
        std::lock_guard<std::mutex> lk(m_);
        closed_ = true;
        not_empty_.notify_all();
        not_full_.notify_all();
    }

    std::size_t size() const {
        std::lock_guard<std::mutex> lk(m_);
        return q_.size();
    }

    std::size_t capacity() const { return capacity_; }

private:
    // O-004（死锁自由）：全类仅此一把锁，每个方法至多持锁一次，无嵌套加锁。
    mutable std::mutex m_;
    std::condition_variable not_empty_;   // popper 等待
    std::condition_variable not_full_;    // pusher 等待
    std::deque<T> q_;                     // O-001：不通过任何 API 暴露
    std::size_t capacity_;
    bool closed_ = false;
};
