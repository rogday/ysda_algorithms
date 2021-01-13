#pragma once

#include <initializer_list>
#include <algorithm>
#include <utility>

class Deque {
public:
    Deque() : m_array_(nullptr), m_capacity_(0), m_size_(0), m_head_(0), m_tail_(0) {
    }
    Deque(const Deque &rhs) : Deque() {
        for (size_t i = 0; i < rhs.Size(); ++i) {
            PushBack(rhs[i]);
        }
    }
    Deque(Deque &&rhs) : Deque() {
        Swap(rhs);
    }
    explicit Deque(size_t size) : Deque() {
        for (size_t i = 0; i < size; ++i) {
            PushBack(0);
        }
    }

    Deque(std::initializer_list<int> list) : Deque() {
        for (const auto &e : list) {
            PushBack(e);
        }
    }

    Deque &operator=(Deque rhs) {
        Swap(rhs);
        return *this;
    }

    void Swap(Deque &rhs) {
        std::swap(m_array_, rhs.m_array_);
        std::swap(m_capacity_, rhs.m_capacity_);
        std::swap(m_size_, rhs.m_size_);
        std::swap(m_head_, rhs.m_head_);
        std::swap(m_tail_, rhs.m_tail_);
    }

    void Reallocate() {
        if (m_head_ != m_tail_ && ((GetBucket(m_head_) == GetBucket(m_tail_)) ||
                                   (GetBucket(MvBwd(m_head_)) != GetBucket(m_tail_)))) {
            return;
        }
        size_t new_cap = std::max(static_cast<size_t>(2), m_capacity_ * 2);
        int **new_arr = new int *[new_cap] {};
        for (size_t j = 0, i = GetBucket(m_head_); i != GetBucket(m_tail_);
             i = (i + 1) % m_capacity_, ++j) {
            new_arr[j] = m_array_[i];
        }
        delete[] m_array_;
        m_array_ = new_arr;
        m_head_ = GetIdx(m_head_);
        m_tail_ = m_size_ + m_head_;
        m_capacity_ = new_cap;
    }

    void PushBack(int value) {
        Reallocate();
        if (GetBucket(MvBwd(m_tail_)) != GetBucket(m_tail_)) {
            AllocBucket(GetBucket(m_tail_));
        }
        m_array_[GetBucket(m_tail_)][GetIdx(m_tail_)] = value;
        m_tail_ = MvFwd(m_tail_);
        ++m_size_;
    }

    void PushFront(int value) {
        Reallocate();
        if (GetBucket(m_head_) != GetBucket(MvBwd(m_head_))) {
            AllocBucket(GetBucket(MvBwd(m_head_)));
        }
        m_array_[GetBucket(MvBwd(m_head_))][GetIdx(MvBwd(m_head_))] = value;
        m_head_ = MvBwd(m_head_);
        ++m_size_;
    }

    void PopBack() {
        if (GetBucket(MvBwd(MvBwd(m_tail_))) != GetBucket(MvBwd(m_tail_))) {
            DeallocBucket(GetBucket(MvBwd(m_tail_)));
        }
        m_tail_ = MvBwd(m_tail_);
        --m_size_;
    }

    void PopFront() {
        if (GetBucket(MvFwd(m_head_)) != GetBucket(m_head_)) {
            DeallocBucket(GetBucket(m_head_));
        }
        m_head_ = MvFwd(m_head_);
        --m_size_;
    }

    int &operator[](size_t ind) {
        size_t new_idx = (m_head_ + ind) % (m_capacity_ * BLOCK_SIZE);
        return m_array_[GetBucket(new_idx)][GetIdx(new_idx)];
    }

    int operator[](size_t ind) const {
        size_t new_idx = (m_head_ + ind) % (m_capacity_ * BLOCK_SIZE);
        return m_array_[GetBucket(new_idx)][GetIdx(new_idx)];
    }

    size_t Size() const {
        return m_size_;
    }

    void Clear() {
        m_size_ = m_head_ = m_tail_ = 0;
        for (size_t i = 0; i < m_capacity_; ++i) {
            DeallocBucket(i);
        }
    }

    ~Deque() {
        Clear();
        delete[] m_array_;
    }

private:
    static constexpr size_t BLOCK_SIZE = 128;

    void AllocBucket(size_t idx) {
        m_array_[idx] = new int[BLOCK_SIZE];
    }

    void DeallocBucket(size_t idx) {
        delete m_array_[idx];
        m_array_[idx] = nullptr;
    }

    size_t MvBwd(size_t val) const {
        return (m_capacity_ * BLOCK_SIZE + val - 1) % (m_capacity_ * BLOCK_SIZE);
    }

    size_t MvFwd(size_t val) const {
        return (val + 1) % (m_capacity_ * BLOCK_SIZE);
    }

    size_t GetBucket(size_t val) const {
        return val / BLOCK_SIZE;
    }

    size_t GetIdx(size_t val) const {
        return val % BLOCK_SIZE;
    }

public:
    int **m_array_;
    size_t m_capacity_;  // for array
    size_t m_size_;      // for deque
    size_t m_head_;
    size_t m_tail_;
};
