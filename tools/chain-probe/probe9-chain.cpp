// Vendored upstream chain_points harness (same-input comparison probe).
#define EPSILON 1e-4
#define SCALED_EPSILON 100.0
#include <algorithm>
#include <array>
#include <cassert>
#include <cmath>
#include <cstdint>
#include <iostream>
#include <limits>
#include <type_traits>
#include <vector>
struct Vec2d {
    double x = 0, y = 0;
    Vec2d() = default;
    Vec2d(double x_, double y_) : x(x_), y(y_) {}
    template<class T> Vec2d cast() const { return *this; }
    double operator[](size_t i) const { return i == 0 ? x : y; }
    double &operator[](size_t i) { return i == 0 ? x : y; }
    Vec2d operator-(const Vec2d &o) const { return Vec2d(x - o.x, y - o.y); }
    double squaredNorm() const { return x * x + y * y; }
};
#ifndef slic3r_MutablePriorityQueue_hpp_
#define slic3r_MutablePriorityQueue_hpp_

#include <assert.h>
#include <type_traits>
constexpr auto InvalidQueueID = std::numeric_limits<size_t>::max();
template<typename T, typename IndexSetter, typename LessPredicate, const bool ResetIndexWhenRemoved = false>
class MutablePriorityQueue
{
public:
	static_assert(std::is_trivially_copyable<T>::value, "Template argument T must be a trivially copiable type in class template MutablePriorityQueue");

	// It is recommended to use make_mutable_priority_queue() for construction.
	MutablePriorityQueue(IndexSetter &&index_setter, LessPredicate &&less_predicate) :
		m_index_setter(std::forward<IndexSetter>(index_setter)), 
		m_less_predicate(std::forward<LessPredicate>(less_predicate)) 
		{}
	~MutablePriorityQueue()	{ clear(); }

	void		clear();
	void		reserve(size_t cnt) 				{ m_heap.reserve(cnt); }
	void		push(const T &item);
	void		push(T &&item);
	void		pop();
	T&			top()								{ return m_heap.front(); }
	void		remove(size_t idx);
	void		update(size_t idx) 					{ T item = m_heap[idx]; remove(idx); push(item); }

	size_t		size() const						{ return m_heap.size(); }
	bool		empty() const						{ return m_heap.empty(); }
	T&			operator[](std::size_t idx) noexcept { return m_heap[idx]; }
	const T&	operator[](std::size_t idx) const noexcept { return m_heap[idx]; }

	using iterator		 = typename std::vector<T>::iterator;
	using const_iterator = typename std::vector<T>::const_iterator;
	iterator 		begin() 		{ return m_heap.begin(); }
	iterator 		end() 			{ return m_heap.end(); }
	const_iterator 	cbegin() const	{ return m_heap.cbegin(); }
	const_iterator 	cend() const	{ return m_heap.cend(); }

protected:
	void		update_heap_up(size_t top, size_t bottom);
	void		update_heap_down(size_t top, size_t bottom);

private:
	std::vector<T>	m_heap;
	IndexSetter		m_index_setter;
	LessPredicate	m_less_predicate;
};

template<typename T, const bool ResetIndexWhenRemoved, typename IndexSetter, typename LessPredicate>
MutablePriorityQueue<T, IndexSetter, LessPredicate, ResetIndexWhenRemoved> make_mutable_priority_queue(IndexSetter &&index_setter, LessPredicate &&less_predicate)
{
    return MutablePriorityQueue<T, IndexSetter, LessPredicate, ResetIndexWhenRemoved>(
    	std::forward<IndexSetter>(index_setter), std::forward<LessPredicate>(less_predicate));
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::clear()
{ 
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		for (size_t idx = 0; idx < m_heap.size(); ++ idx)
			// Mark as removed from the queue.
			m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
	}
	m_heap.clear();
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::push(const T &item)
{
	size_t idx = m_heap.size();
	m_heap.emplace_back(item);
	m_index_setter(m_heap.back(), idx);
	update_heap_up(0, idx);
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::push(T &&item)
{
	size_t idx = m_heap.size();
	m_heap.emplace_back(std::move(item));
	m_index_setter(m_heap.back(), idx);
	update_heap_up(0, idx);
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::pop()
{
	assert(! m_heap.empty());
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		// Mark as removed from the queue.
		m_index_setter(m_heap.front(), std::numeric_limits<size_t>::max());
	}
	if (m_heap.size() > 1) {
		m_heap.front() = m_heap.back();
		m_heap.pop_back();
		m_index_setter(m_heap.front(), 0);
		update_heap_down(0, m_heap.size() - 1);
	} else
		m_heap.clear();
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::remove(size_t idx)
{
	assert(idx < m_heap.size());
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		// Mark as removed from the queue.
		m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
	}
	if (idx + 1 == m_heap.size()) {
		m_heap.pop_back();
		return;
	}
	m_heap[idx] = m_heap.back();
	m_index_setter(m_heap[idx], idx);
	m_heap.pop_back();
	update_heap_down(idx, m_heap.size() - 1);
	update_heap_up(0, idx);
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::update_heap_up(size_t top, size_t bottom)
{
	size_t childIdx = bottom;
	T *child = &m_heap[childIdx];
	for (;;) {
		size_t parentIdx = (childIdx - 1) >> 1;
		if (childIdx == 0 || parentIdx < top)
			break;
		T *parent = &m_heap[parentIdx];
		// switch nodes
		if (! m_less_predicate(*parent, *child)) {
			T tmp = *parent;
			m_index_setter(tmp,    childIdx);
			m_index_setter(*child, parentIdx);
			m_heap[parentIdx] = *child;
			m_heap[childIdx]  = tmp;
		}
		// shift up
		childIdx = parentIdx;
		child = parent;
	}
}

template<class T, class LessPredicate, class IndexSetter, const bool ResetIndexWhenRemoved>
inline void MutablePriorityQueue<T, LessPredicate, IndexSetter, ResetIndexWhenRemoved>::update_heap_down(size_t top, size_t bottom)
{
	size_t parentIdx = top;
	T *parent = &m_heap[parentIdx];
	for (;;) {
		size_t childIdx = (parentIdx << 1) + 1;
		if (childIdx > bottom)
			break;
		T *child = &m_heap[childIdx];
		size_t child2Idx = childIdx + 1;
		if (child2Idx <= bottom) {
			T *child2 = &m_heap[child2Idx];
			if (! m_less_predicate(*child, *child2)) {
				child = child2;
				childIdx = child2Idx;
			}
		}
		if (m_less_predicate(*parent, *child))
			return;
		// switch nodes
		T tmp = *parent;
		m_index_setter(tmp,    childIdx);
		m_index_setter(*child, parentIdx);
		m_heap[parentIdx] = *child;
		m_heap[childIdx] = tmp;
		// shift down
		parentIdx = childIdx;
		parent = child;
	}
}

// Binary heap addressing of a hierarchy of binary miniheaps by a higher level binary heap.
// Conceptually it works the same as a plain binary heap, however it is cache friendly.
// A binary block of "block_size" implements a binary miniheap of (block_size / 2) leaves and 
// ((block_size / 2) - 1) nodes, thus wasting a single element. To make addressing simpler,
// the zero'th element inside each miniheap is wasted, thus for example a single element heap is
// 2 elements long and the 1st element starts at address 1.
//
// Mostly copied from the following great source:
// https://playfulprogramming.blogspot.com/2015/08/cache-optimizing-priority-queue.html
// https://github.com/rollbear/prio_queue/blob/master/prio_queue.hpp
// original source Copyright Björn Fahller 2015, Boost Software License, Version 1.0, http://www.boost.org/LICENSE_1_0.txt
template <std::size_t blocking>
struct SkipHeapAddressing
{
public:
	static const constexpr std::size_t block_size = blocking;
	static const constexpr std::size_t block_mask = block_size - 1;
	static_assert((block_size & block_mask) == 0U, "block size must be 2^n for some integer n");

	static inline std::size_t child_of(std::size_t node_no) noexcept {
  		if (! is_block_leaf(node_no))
  			// If not a leaf, then it is sufficient to just traverse down inside a miniheap.
  			// The following line is equivalent to, but quicker than
  			// return block_base(node_no) + 2 * block_offset(node_no);
			return node_no + block_offset(node_no);
		// Otherwise skip to a root of a child miniheap.
		return (block_base(node_no) + 1 + child_no(node_no) * 2) * block_size + 1;
	}

	static inline std::size_t parent_of(std::size_t node_no) noexcept {
  		auto const node_root = block_base(node_no); // 16
  		if (! is_block_root(node_no))
  			// If not a block (miniheap) root, then it is sufficient to just traverse up inside a miniheap.
			return node_root + block_offset(node_no) / 2;
		// Otherwise skipping from a root of one miniheap into leaf of another miniheap.
		// Address of a parent miniheap block. One miniheap branches at (block_size / 2) leaves to (block_size) miniheaps.
		auto const parent_base = block_base(node_root / block_size - 1); // 0
		// Index of a leaf of a parent miniheap, which is a parent of node_no.
		auto const child       = ((node_no - block_size) / block_size - parent_base) / 2;
		return 
			// Address of a parent miniheap
			parent_base + 
			// Address of a leaf of a parent miniheap
			block_size / 2 + child; // 30
	}

	// Leafs are stored inside the second half of a block.
	static inline bool 			is_block_leaf(std::size_t node_no) noexcept { return (node_no & (block_size >> 1)) != 0U; }
	// Unused space aka padding to facilitate quick addressing.
	static inline bool 			is_padding   (std::size_t node_no) noexcept { return block_offset(node_no) == 0U; }
// Following methods are internal, but made public for unit tests.
//private:
	// Address is a root of a block (of a miniheap).
	static inline bool 			is_block_root(std::size_t node_no) noexcept { return block_offset(node_no) == 1U; }
	// Offset inside a block (inside a miniheap).
	static inline std::size_t 	block_offset (std::size_t node_no) noexcept { return node_no & block_mask; }
	// Base address of a block (a miniheap).
	static inline std::size_t 	block_base   (std::size_t node_no) noexcept { return node_no & ~block_mask; }
	// Index of a leaf.
	static inline std::size_t 	child_no     (std::size_t node_no) noexcept { assert(is_block_leaf(node_no)); return node_no & (block_mask >> 1); }
};

// Cache friendly variant of MutablePriorityQueue, implemented as a binary heap of binary miniheaps,
// building upon SkipHeapAddressing.
template<typename T, typename IndexSetter, typename LessPredicate, std::size_t blocking = 32, const bool ResetIndexWhenRemoved = false>
class MutableSkipHeapPriorityQueue
{
public:
	static_assert(std::is_trivially_copyable<T>::value, "Template argument T must be a trivially copiable type in class template MutableSkipHeapPriorityQueue");
	using address = SkipHeapAddressing<blocking>;

	// It is recommended to use make_miniheap_mutable_priority_queue() for construction.
	MutableSkipHeapPriorityQueue(IndexSetter &&index_setter, LessPredicate &&less_predicate) :
		m_index_setter(std::forward<IndexSetter>(index_setter)), 
		m_less_predicate(std::forward<LessPredicate>(less_predicate)) 
		{}
	~MutableSkipHeapPriorityQueue()	{ clear(); }

	void		clear();
	// Reserve one unused element per miniheap.
	void		reserve(size_t cnt) 				{ m_heap.reserve(cnt + ((cnt + (address::block_size - 1)) / (address::block_size - 1))); }
	void		push(const T &item);
	void		push(T &&item);
	void		pop();
	T&			top()								{ return m_heap[1]; }
	void		remove(size_t idx);
	void		update(size_t idx) 					{ assert(! address::is_padding(idx)); T item = m_heap[idx]; remove(idx); push(item); }
	// There is one padding element storead at each miniheap, thus lower the number of elements by the number of miniheaps.
	size_t 		size() const noexcept 				{ return m_heap.size() - (m_heap.size() + address::block_size - 1) / address::block_size; }
	bool		empty() const						{ return m_heap.empty(); }
	T&			operator[](std::size_t idx) noexcept { assert(! address::is_padding(idx)); return m_heap[idx]; }
	const T&    operator[](std::size_t idx) const noexcept { assert(! address::is_padding(idx)); return m_heap[idx]; }

protected:
	void		update_heap_up(size_t top, size_t bottom);
	void		update_heap_down(size_t top, size_t bottom);
	void   		pop_back() noexcept {
		assert(m_heap.size() > 1);
		assert(! address::is_padding(m_heap.size() - 1));
		m_heap.pop_back();
		if (address::is_padding(m_heap.size() - 1))
			m_heap.pop_back();
	}

private:
	std::vector<T>	m_heap;
	IndexSetter		m_index_setter;
	LessPredicate	m_less_predicate;
};

template<typename T, std::size_t BlockSize, const bool ResetIndexWhenRemoved, typename IndexSetter, typename LessPredicate>
MutableSkipHeapPriorityQueue<T, IndexSetter, LessPredicate, BlockSize, ResetIndexWhenRemoved> 
	make_miniheap_mutable_priority_queue(IndexSetter &&index_setter, LessPredicate &&less_predicate)
{
    return MutableSkipHeapPriorityQueue<T, IndexSetter, LessPredicate, BlockSize, ResetIndexWhenRemoved>(
    	std::forward<IndexSetter>(index_setter), std::forward<LessPredicate>(less_predicate));
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::clear()
{ 
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		for (size_t idx = 0; idx < m_heap.size(); ++ idx)
			// Mark as removed from the queue.
			if (! address::is_padding(idx))
				m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
	}
	m_heap.clear();
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::push(const T &item)
{
	if (address::is_padding(m_heap.size()))
		m_heap.emplace_back(T());
	size_t idx = m_heap.size();
	m_heap.emplace_back(item);
	m_index_setter(m_heap.back(), idx);
	update_heap_up(1, idx);
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::push(T &&item)
{
	if (address::is_padding(m_heap.size()))
		m_heap.emplace_back(T());
	size_t idx = m_heap.size();
	m_heap.emplace_back(std::move(item));
	m_index_setter(m_heap.back(), idx);
	update_heap_up(1, idx);
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::pop()
{
	assert(! m_heap.empty());
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		// Mark as removed from the queue.
		m_index_setter(m_heap[1], std::numeric_limits<size_t>::max());
	}
	// Zero'th element is padding, thus non-empty queue must have at least two elements.
	if (m_heap.size() > 2) {
		m_heap[1] = m_heap.back();
		this->pop_back();
		m_index_setter(m_heap[1], 1);
		update_heap_down(1, m_heap.size() - 1);
	} else
		m_heap.clear();
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::remove(size_t idx)
{
	assert(idx < m_heap.size());
	assert(! address::is_padding(idx));
#ifdef NDEBUG
	// Only mark as removed from the queue in release mode, if configured so.
	if (ResetIndexWhenRemoved)
#endif /* NDEBUG */
	{
		// Mark as removed from the queue.
		m_index_setter(m_heap[idx], std::numeric_limits<size_t>::max());
	}
	if (idx + 1 == m_heap.size()) {
		this->pop_back();
		return;
	}
	m_heap[idx] = m_heap.back();
	m_index_setter(m_heap[idx], idx);
	this->pop_back();
	update_heap_down(idx, m_heap.size() - 1);
	update_heap_up(1, idx);
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::update_heap_up(size_t top, size_t bottom)
{
	assert(! address::is_padding(top));
	assert(! address::is_padding(bottom));
	size_t childIdx = bottom;
	T *child = &m_heap[childIdx];
	for (;;) {
		size_t parentIdx = address::parent_of(childIdx);
		if (childIdx == 1 || parentIdx < top)
			break;
		T *parent = &m_heap[parentIdx];
		// switch nodes
		if (! m_less_predicate(*parent, *child)) {
			T tmp = *parent;
			m_index_setter(tmp,    childIdx);
			m_index_setter(*child, parentIdx);
			m_heap[parentIdx] = *child;
			m_heap[childIdx]  = tmp;
		}
		// shift up
		childIdx = parentIdx;
		child = parent;
	}
}

template<class T, class LessPredicate, class IndexSetter, std::size_t blocking, const bool ResetIndexWhenRemoved>
inline void MutableSkipHeapPriorityQueue<T, LessPredicate, IndexSetter, blocking, ResetIndexWhenRemoved>::update_heap_down(size_t top, size_t bottom)
{
	assert(! address::is_padding(top));
	assert(! address::is_padding(bottom));
	size_t parentIdx = top;
	T *parent = &m_heap[parentIdx];
	for (;;) {
		size_t childIdx = address::child_of(parentIdx);
		if (childIdx > bottom)
			break;
		T *child = &m_heap[childIdx];
		size_t child2Idx = childIdx + (address::is_block_leaf(parentIdx) ? address::block_size : 1);
		if (child2Idx <= bottom) {
			T *child2 = &m_heap[child2Idx];
			if (! m_less_predicate(*child, *child2)) {
				child = child2;
				childIdx = child2Idx;
			}
		}
		if (m_less_predicate(*parent, *child))
			return;
		// switch nodes
		T tmp = *parent;
		m_index_setter(tmp,    childIdx);
		m_index_setter(*child, parentIdx);
		m_heap[parentIdx] = *child;
		m_heap[childIdx]  = tmp;
		// shift down
		parentIdx = childIdx;
		parent = child;
	}
}

#endif /* slic3r_MutablePriorityQueue_hpp_ */
// KD tree built upon external data set, referencing the external data by integer indices.

#ifndef slic3r_KDTreeIndirect_hpp_
#define slic3r_KDTreeIndirect_hpp_

#include <algorithm>
#include <limits>
#include <vector>

#include "Utils.hpp" // for next_highest_power_of_2()

namespace Slic3r {

enum class VisitorReturnMask : unsigned int {
    CONTINUE_LEFT  = 1,
    CONTINUE_RIGHT = 2,
    STOP           = 4,
};

// KD tree for N-dimensional closest point search.
template<size_t ANumDimensions, typename ACoordType, typename ACoordinateFn>
class KDTreeIndirect
{
public:
    static constexpr size_t NumDimensions = ANumDimensions;
    using					CoordinateFn  = ACoordinateFn;
    using					CoordType     = ACoordType;
    // Following could be static constexpr size_t, but that would not link in C++11
    enum : size_t {
        npos = size_t(-1)
    };

    KDTreeIndirect(CoordinateFn coordinate) : coordinate(coordinate) {}
    KDTreeIndirect(CoordinateFn coordinate, std::vector<size_t> indices) : coordinate(coordinate) { this->build(indices); }
    KDTreeIndirect(CoordinateFn coordinate, size_t num_indices) : coordinate(coordinate) { this->build(num_indices); }
    KDTreeIndirect(KDTreeIndirect &&rhs) : m_nodes(std::move(rhs.m_nodes)), coordinate(std::move(rhs.coordinate)) {}
    KDTreeIndirect& operator=(KDTreeIndirect &&rhs) { m_nodes = std::move(rhs.m_nodes); coordinate = std::move(rhs.coordinate); return *this; }
    void clear() { m_nodes.clear(); }

    void build(size_t num_indices)
    {
        std::vector<size_t> indices;
        indices.reserve(num_indices);
        for (size_t i = 0; i < num_indices; ++ i)
            indices.emplace_back(i);
        this->build(indices);
    }

    void build(std::vector<size_t> &indices)
    {
        if (indices.empty())
            clear();
        else {
            // Allocate enough memory for a full binary tree.
            m_nodes.assign(next_highest_power_of_2(indices.size() + 1), npos);
            build_recursive(indices, 0, 0, 0, indices.size() - 1);
        }
        indices.clear();
    }

    template<typename CoordType>
    unsigned int descent_mask(const CoordType &point_coord, const CoordType &search_radius, size_t idx, size_t dimension) const
    {
        CoordType dist = point_coord - this->coordinate(idx, dimension);
        return (dist * dist < search_radius + CoordType(EPSILON)) ?
                                                                    // The plane intersects a hypersphere centered at point_coord of search_radius.
                   ((unsigned int)(VisitorReturnMask::CONTINUE_LEFT) | (unsigned int)(VisitorReturnMask::CONTINUE_RIGHT)) :
                   // The plane does not intersect the hypersphere.
                   (dist > CoordType(0)) ? (unsigned int)(VisitorReturnMask::CONTINUE_RIGHT) : (unsigned int)(VisitorReturnMask::CONTINUE_LEFT);
    }

       // Visitor is supposed to return a bit mask of VisitorReturnMask.
    template<typename Visitor>
    void visit(Visitor &visitor) const
    {
        visit_recursive(0, 0, visitor);
    }

    CoordinateFn coordinate;

private:
    // Build a balanced tree by splitting the input sequence by an axis aligned plane at a dimension.
    void build_recursive(std::vector<size_t> &input, size_t node, const size_t dimension, const size_t left, const size_t right)
    {
        if (left > right)
            return;

        assert(node < m_nodes.size());

        if (left == right) {
            // Insert a node into the balanced tree.
            m_nodes[node] = input[left];
            return;
        }

           // Partition the input to left / right pieces of the same length to produce a balanced tree.
        size_t center = (left + right) / 2;
        partition_input(input, dimension, left, right, center);
        // Insert a node into the tree.
        m_nodes[node] = input[center];
        // Build up the left / right subtrees.
        size_t next_dimension = dimension;
        if (++ next_dimension == NumDimensions)
            next_dimension = 0;
        if (center > left)
            build_recursive(input, node * 2 + 1, next_dimension, left, center - 1);
        build_recursive(input, node * 2 + 2, next_dimension, center + 1, right);
    }

       // Partition the input m_nodes <left, right> at "k" and "dimension" using the QuickSelect method:
       // https://en.wikipedia.org/wiki/Quickselect
       // Items left of the k'th item are lower than the k'th item in the "dimension",
       // items right of the k'th item are higher than the k'th item in the "dimension",
    void partition_input(std::vector<size_t> &input, const size_t dimension, size_t left, size_t right, const size_t k) const
    {
        while (left < right) {
            size_t center = (left + right) / 2;
            CoordType pivot;
            {
                // Bubble sort the input[left], input[center], input[right], so that a median of the three values
                // will end up in input[center].
                CoordType left_value   = this->coordinate(input[left],   dimension);
                CoordType center_value = this->coordinate(input[center], dimension);
                CoordType right_value  = this->coordinate(input[right],  dimension);
                if (left_value > center_value) {
                    std::swap(input[left], input[center]);
                    std::swap(left_value,  center_value);
                }
                if (left_value > right_value) {
                    std::swap(input[left], input[right]);
                    right_value = left_value;
                }
                if (center_value > right_value) {
                    std::swap(input[center], input[right]);
                    center_value = right_value;
                }
                pivot = center_value;
            }
            if (right <= left + 2)
                // The <left, right> interval is already sorted.
                break;
            size_t i = left;
            size_t j = right - 1;
            std::swap(input[center], input[j]);
            // Partition the set based on the pivot.
            for (;;) {
                // Skip left points that are already at correct positions.
                // Search will certainly stop at position (right - 1), which stores the pivot.
                while (this->coordinate(input[++ i], dimension) < pivot) ;
                // Skip right points that are already at correct positions.
                while (this->coordinate(input[-- j], dimension) > pivot && i < j) ;
                if (i >= j)
                    break;
                std::swap(input[i], input[j]);
            }
            // Restore pivot to the center of the sequence.
            std::swap(input[i], input[right - 1]);
            // Which side the kth element is in?
            if (k < i)
                right = i - 1;
            else if (k == i)
                // Sequence is partitioned, kth element is at its place.
                break;
            else
                left = i + 1;
        }
    }

    template<typename Visitor>
    void visit_recursive(size_t node, size_t dimension, Visitor &visitor) const
    {
        assert(! m_nodes.empty());
        if (node >= m_nodes.size() || m_nodes[node] == npos)
            return;

           // Left / right child node index.
        size_t left  = node * 2 + 1;
        size_t right = left + 1;
        unsigned int mask = visitor(m_nodes[node], dimension);
        if ((mask & (unsigned int)VisitorReturnMask::STOP) == 0) {
            size_t next_dimension = (++ dimension == NumDimensions) ? 0 : dimension;
            if (mask & (unsigned int)VisitorReturnMask::CONTINUE_LEFT)
                visit_recursive(left,  next_dimension, visitor);
            if (mask & (unsigned int)VisitorReturnMask::CONTINUE_RIGHT)
                visit_recursive(right, next_dimension, visitor);
        }
    }

    std::vector<size_t> m_nodes;
};

// Find a closest point using Euclidian metrics.
// Returns npos if not found.
template<size_t K,
         typename PointType,
         typename FilterFn,
         size_t D,
         typename CoordT,
         typename CoordFn>
std::array<size_t, K> find_closest_points(
    const KDTreeIndirect<D, CoordT, CoordFn> &kdtree,
    const PointType                          &point,
    FilterFn                                  filter)
{
    using Tree = KDTreeIndirect<D, CoordT, CoordFn>;

    struct Visitor
    {
        const Tree      &kdtree;
        const PointType &point;
        const FilterFn   filter;

        std::array<std::pair<size_t, CoordT>, K> results;

        Visitor(const Tree &kdtree, const PointType &point, FilterFn filter)
            : kdtree(kdtree), point(point), filter(filter)
        {
            results.fill(std::make_pair(Tree::npos,
                                        std::numeric_limits<CoordT>::max()));
        }
        unsigned int operator()(size_t idx, size_t dimension)
        {
            if (this->filter(idx)) {
                auto dist = CoordT(0);
                for (size_t i = 0; i < D; ++i) {
                    CoordT d = point[i] - kdtree.coordinate(idx, i);
                    dist += d * d;
                }

                auto res = std::make_pair(idx, dist);
                auto it  = std::lower_bound(results.begin(), results.end(),
                                            res, [](auto &r1, auto &r2) {
                                               return r1.second < r2.second;
                                            });

                if (it != results.end()) {
                    std::rotate(it, std::prev(results.end()), results.end());
                    *it = res;
                }
            }
            return kdtree.descent_mask(point[dimension],
                                       results.front().second, idx,
                                       dimension);
        }
    } visitor(kdtree, point, filter);

    kdtree.visit(visitor);
    std::array<size_t, K> ret;
    for (size_t i = 0; i < K; i++) ret[i] = visitor.results[i].first;

    return ret;
}

template<size_t K, typename PointType, size_t D, typename CoordT, typename CoordFn>
std::array<size_t, K> find_closest_points(
    const KDTreeIndirect<D, CoordT, CoordFn> &kdtree, const PointType &point)
{
    return find_closest_points<K>(kdtree, point, [](size_t) { return true; });
}

template<typename PointType,
         typename FilterFn,
         size_t D,
         typename CoordT,
         typename CoordFn>
size_t find_closest_point(const KDTreeIndirect<D, CoordT, CoordFn> &kdtree,
                          const PointType                          &point,
                          FilterFn                                  filter)
{
    return find_closest_points<1>(kdtree, point, filter)[0];
}

template<typename KDTreeIndirectType, typename PointType>
size_t find_closest_point(const KDTreeIndirectType& kdtree, const PointType& point)
{
    return find_closest_point(kdtree, point, [](size_t) { return true; });
}

// Find nearby points (spherical neighbourhood) using Euclidian metrics.
template<typename KDTreeIndirectType, typename PointType, typename FilterFn>
std::vector<size_t> find_nearby_points(const KDTreeIndirectType &kdtree, const PointType &center,
                                       const typename KDTreeIndirectType::CoordType& max_distance, FilterFn filter)
{
    using CoordType = typename KDTreeIndirectType::CoordType;

    struct Visitor {
        const KDTreeIndirectType &kdtree;
        const PointType center;
        const CoordType max_distance_squared;
        const FilterFn filter;
        std::vector<size_t> result;

        Visitor(const KDTreeIndirectType &kdtree, const PointType& center, const CoordType &max_distance,
                FilterFn filter) :
            kdtree(kdtree), center(center), max_distance_squared(max_distance*max_distance), filter(filter) {
        }
        unsigned int operator()(size_t idx, size_t dimension) {
            if (this->filter(idx)) {
                auto dist = CoordType(0);
                for (size_t i = 0; i < KDTreeIndirectType::NumDimensions; ++i) {
                    CoordType d = center[i] - kdtree.coordinate(idx, i);
                    dist += d * d;
                }
                if (dist < max_distance_squared) {
                    result.push_back(idx);
                }
            }
            return kdtree.descent_mask(center[dimension], max_distance_squared, idx, dimension);
        }
    } visitor(kdtree, center, max_distance, filter);

    kdtree.visit(visitor);
    return visitor.result;
}

template<typename KDTreeIndirectType, typename PointType>
std::vector<size_t> find_nearby_points(const KDTreeIndirectType &kdtree, const PointType &center,
                                       const typename KDTreeIndirectType::CoordType& max_distance)
{
    return find_nearby_points(kdtree, center, max_distance, [](size_t) {
        return true;
    });
}

// Find nearby points (spherical neighbourhood) using Euclidian metrics.
template<typename KDTreeIndirectType, typename PointType, typename FilterFn>
std::vector<size_t> find_nearby_points(const KDTreeIndirectType &kdtree,
                                       const PointType          &bb_min,
                                       const PointType          &bb_max,
                                       FilterFn                  filter)
{
    struct Visitor {
        const KDTreeIndirectType &kdtree;
        const PointType &bb_min, &bb_max;
        const FilterFn filter;
        std::vector<size_t> result;

        Visitor(const KDTreeIndirectType &kdtree, const PointType& bbmin, const PointType& bbmax,
                FilterFn filter) :
            kdtree(kdtree), bb_min{bbmin}, bb_max{bbmax}, filter(filter) {
        }
        unsigned int operator()(size_t idx, size_t dimension) {
            unsigned int ret =
                static_cast<unsigned int>(VisitorReturnMask::CONTINUE_LEFT) |
                static_cast<unsigned int>(VisitorReturnMask::CONTINUE_RIGHT);

            if (this->filter(idx)) {
                PointType p;
                bool contains = true;
                for (size_t i = 0; i < KDTreeIndirectType::NumDimensions; ++i) {
                    p(i) = kdtree.coordinate(idx, i);
                    contains = contains && bb_min(i) <= p(i) && p(i) <= bb_max(i);
                }

                if (p(dimension) < bb_min(dimension))
                    ret = static_cast<unsigned int>(VisitorReturnMask::CONTINUE_RIGHT);
                if (p(dimension) > bb_max(dimension))
                    ret = static_cast<unsigned int>(VisitorReturnMask::CONTINUE_LEFT);

                if (contains)
                    result.emplace_back(idx);
            }

            return ret;
        }
    } visitor(kdtree, bb_min, bb_max, filter);

    kdtree.visit(visitor);
    return visitor.result;
}

} // namespace Slic3r

#endif /* slic3r_KDTreeIndirect_hpp_ */
using namespace Slic3r;
namespace vendored {
// This implementation will always produce valid result even if some segments cannot reverse.
template<typename EndPointType, typename KDTreeType, typename CouldReverseFunc>
std::vector<std::pair<size_t, bool>> chain_segments_closest_point(std::vector<EndPointType> &end_points, KDTreeType &kdtree, CouldReverseFunc &could_reverse_func, EndPointType &first_point)
{
	assert((end_points.size() & 1) == 0);
    size_t num_segments = end_points.size() / 2;
	assert(num_segments >= 2);
	for (EndPointType &ep : end_points)
		ep.chain_id = 0;
	std::vector<std::pair<size_t, bool>> out;
	out.reserve(num_segments);
	size_t first_point_idx = &first_point - end_points.data();
	out.emplace_back(first_point_idx / 2, (first_point_idx & 1) != 0);
	first_point.chain_id = 1;
	size_t this_idx = first_point_idx ^ 1;
	for (int iter = (int)num_segments - 2; iter >= 0; -- iter) {
		EndPointType &this_point = end_points[this_idx];
    	this_point.chain_id = 1;
    	// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the lambda).
    	// Ignore the starting point as the starting point is considered to be occupied, no end point coud connect to it.
		size_t next_idx = find_closest_point(kdtree, this_point.pos,
			[this_idx, &end_points, &could_reverse_func](size_t idx) {
				return (idx ^ this_idx) > 1 && end_points[idx].chain_id == 0 && ((idx & 1) == 0 || could_reverse_func(idx >> 1));
		});
		assert(next_idx < end_points.size());
		EndPointType &end_point = end_points[next_idx];
		end_point.chain_id = 1;
		assert((next_idx & 1) == 0 || could_reverse_func(next_idx >> 1));
		out.emplace_back(next_idx / 2, (next_idx & 1) != 0);
		this_idx = next_idx ^ 1;
	}
#ifndef NDEBUG
	assert(end_points[this_idx].chain_id == 0);
	for (EndPointType &ep : end_points)
		assert(&ep == &end_points[this_idx] || ep.chain_id == 1);
#endif /* NDEBUG */
	return out;
}

// Chain perimeters (always closed) and thin fills (closed or open) using a greedy algorithm.
// Solving a Traveling Salesman Problem (TSP) with the modification, that the sites are not always points, but points and segments.
// Solving using a greedy algorithm, where a shortest edge is added to the solution if it does not produce a bifurcation or a cycle.
// Return index and "reversed" flag.
// https://en.wikipedia.org/wiki/Multi-fragment_algorithm
// The algorithm builds a tour for the traveling salesman one edge at a time and thus maintains multiple tour fragments, each of which 
// is a simple path in the complete graph of cities. At each stage, the algorithm selects the edge of minimal cost that either creates 
// a new fragment, extends one of the existing paths or creates a cycle of length equal to the number of cities.
template<typename PointType, typename SegmentEndPointFunc, bool REVERSE_COULD_FAIL, typename CouldReverseFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy_constrained_reversals_(SegmentEndPointFunc end_point_func, CouldReverseFunc could_reverse_func, size_t num_segments, const PointType *start_near)
{
	std::vector<std::pair<size_t, bool>> out;

	if (num_segments == 0) {
		// Nothing to do.
	} 
	else if (num_segments == 1)
	{
		// Just sort the end points so that the first point visited is closest to start_near.
		out.emplace_back(0, could_reverse_func(0) && start_near != nullptr && 
            (end_point_func(0, false) - *start_near).template cast<double>().squaredNorm() < (end_point_func(0, true) - *start_near).template cast<double>().squaredNorm());
	} 
	else
	{
		// End points of segments for the KD tree closest point search.
		// A single end point is inserted into the search structure for loops, two end points are entered for open paths.
		struct EndPoint {
			EndPoint(const Vec2d &pos) : pos(pos) {}
			Vec2d     pos;
			// Identifier of the chain, to which this end point belongs. Zero means unassigned.
			size_t    chain_id = 0;
			// Link to the closest currently valid end point.
			EndPoint *edge_out = nullptr;
			// Distance to the next end point following the link.
			// Zero value -> start of the final path.
			double    distance_out = std::numeric_limits<double>::max();
			size_t    heap_idx = std::numeric_limits<size_t>::max();
		};
	    std::vector<EndPoint> end_points;
	    end_points.reserve(num_segments * 2);
	    for (size_t i = 0; i < num_segments; ++ i) {
            end_points.emplace_back(end_point_func(i, true ).template cast<double>());
            end_points.emplace_back(end_point_func(i, false).template cast<double>());
	    }

	    // Construct the closest point KD tree over end points of segments.
		auto coordinate_fn = [&end_points](size_t idx, size_t dimension) -> double { return end_points[idx].pos[dimension]; };
		KDTreeIndirect<2, double, decltype(coordinate_fn)> kdtree(coordinate_fn, end_points.size());

		// Helper to detect loops in already connected paths.
		// Unique chain IDs are assigned to paths. If paths are connected, end points will not have their chain IDs updated, but the chain IDs
		// will remember an "equivalent" chain ID, which is the lowest ID of all the IDs in the path, and the lowest ID is equivalent to itself.
		class EquivalentChains {
		public:
			// Zero'th chain ID is invalid.
			EquivalentChains(size_t reserve) { m_equivalent_with.reserve(reserve); m_equivalent_with.emplace_back(0); }
			// Generate next equivalence class.
			size_t 				next() {
				m_equivalent_with.emplace_back(++ m_last_chain_id);
				return m_last_chain_id;
			}
			// Get equivalence class for chain ID.
			size_t 				operator()(size_t chain_id) {
				if (chain_id != 0) {
					for (size_t last = chain_id;;) {
						size_t lower = m_equivalent_with[last];
						if (lower == last) {
							m_equivalent_with[chain_id] = lower;
							chain_id = lower;
							break;
						}
						last = lower;
					}
				}
				return chain_id;
			}
			size_t 			  	merge(size_t chain_id1, size_t chain_id2) {
				size_t chain_id = std::min((*this)(chain_id1), (*this)(chain_id2));
				m_equivalent_with[chain_id1] = chain_id;
				m_equivalent_with[chain_id2] = chain_id;
				return chain_id;
			}

#ifndef NDEBUG
			bool				validate()
			{
				assert(m_last_chain_id >= 0);
				assert(m_last_chain_id + 1 == m_equivalent_with.size());
				for (size_t i = 0; i < m_equivalent_with.size(); ++ i) {
					for (size_t last = i;;) {
						size_t lower = m_equivalent_with[last];
						assert(lower <= last);
						if (lower == last)
							break;
						last = lower;
					}
				}
				return true;
			}
#endif /* NDEBUG */

		private:
			// Unique chain ID assigned to chains of end points of segments.
			size_t              m_last_chain_id = 0;
			std::vector<size_t> m_equivalent_with;
		} equivalent_chain(num_segments);

		// Find the first end point closest to start_near.
		EndPoint *first_point = nullptr;
		size_t    first_point_idx = std::numeric_limits<size_t>::max();
		if (start_near != nullptr) {
            size_t idx = find_closest_point(kdtree, start_near->template cast<double>(),
				// Don't start with a reverse segment, if flipping of the segment is not allowed.
				[&could_reverse_func](size_t idx) { return (idx & 1) == 0 || could_reverse_func(idx >> 1); });
			assert(idx < end_points.size());
			first_point = &end_points[idx];
			first_point->distance_out = 0.;
			first_point->chain_id = equivalent_chain.next();
			first_point_idx = idx;
		}
		EndPoint *initial_point = first_point;
		EndPoint *last_point = nullptr;

		// Assign the closest point and distance to the end points.
		for (EndPoint &end_point : end_points) {
	    	assert(end_point.edge_out == nullptr);
	    	if (&end_point != first_point) {
		    	size_t this_idx = &end_point - &end_points.front();
		    	// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the lambda).
		    	// Ignore the starting point as the starting point is considered to be occupied, no end point coud connect to it.
				size_t next_idx = find_closest_point(kdtree, end_point.pos, 
					[this_idx, first_point_idx](size_t idx){ return idx != first_point_idx && (idx ^ this_idx) > 1; });
				assert(next_idx < end_points.size());
				EndPoint &end_point2 = end_points[next_idx];
				end_point.edge_out = &end_point2;
				end_point.distance_out = (end_point2.pos - end_point.pos).squaredNorm();
			}
		}

	    // Initialize a heap of end points sorted by the lowest distance to the next valid point of a path.
	    auto queue = make_mutable_priority_queue<EndPoint*, false>(
			[](EndPoint *ep, size_t idx){ ep->heap_idx = idx; }, 
	    	[](EndPoint *l, EndPoint *r){ return l->distance_out < r->distance_out; });
		queue.reserve(end_points.size() * 2 - 1);
	    for (EndPoint &ep : end_points)
	    	if (first_point != &ep)
				queue.push(&ep);

#ifndef NDEBUG
		auto validate_graph_and_queue = [&equivalent_chain, &end_points, &queue, first_point]() -> bool {
			assert(equivalent_chain.validate());
			for (EndPoint &ep : end_points) {
				if (ep.heap_idx < queue.size()) {
					// End point is on the heap.
					assert(*(queue.cbegin() + ep.heap_idx) == &ep);
					assert(ep.chain_id == 0);
				} else {
					// End point is NOT on the heap, therefore it is part of the output path.
					assert(ep.heap_idx == std::numeric_limits<size_t>::max());
					assert(ep.chain_id != 0);
					if (&ep == first_point) {
						assert(ep.edge_out == nullptr);
					} else {
						assert(ep.edge_out != nullptr);
						// Detect loops.
						for (EndPoint *pt = &ep; pt != nullptr;) {
							// Out of queue. It is a final point.
							assert(pt->heap_idx == std::numeric_limits<size_t>::max());
							EndPoint *pt_other = &end_points[(pt - &end_points.front()) ^ 1];
							if (pt_other->heap_idx < queue.size())
								// The other side of this segment is undecided yet.
								break;
							pt = pt_other->edge_out;
						}
					}
				}
			}
			for (EndPoint *ep : queue)
				// Points in the queue are not connected yet.
				assert(ep->chain_id == 0);
			return true;
		};
#endif /* NDEBUG */

	    // Chain the end points: find (num_segments - 1) shortest links not forming bifurcations or loops.
		assert(num_segments >= 2);
#ifndef NDEBUG
		double distance_taken_last = 0.;
#endif /* NDEBUG */
		for (int iter = int(num_segments) - 2;; -- iter) {
			assert(validate_graph_and_queue());
	    	// Take the first end point, for which the link points to the currently closest valid neighbor.
	    	EndPoint &end_point1 = *queue.top();
#ifndef NDEBUG
			// Each edge added shall be longer than the previous one taken.
			assert(end_point1.distance_out > distance_taken_last - SCALED_EPSILON);
			distance_taken_last = end_point1.distance_out;
#endif /* NDEBUG */
			assert(end_point1.edge_out != nullptr);
	    	// No point on the queue may be connected yet.
	    	assert(end_point1.chain_id == 0);
	    	// Take the closest end point to the first end point,
	    	EndPoint &end_point2 = *end_point1.edge_out;
	    	bool valid = true;
	    	size_t end_point1_other_chain_id = 0;
	    	size_t end_point2_other_chain_id = 0;
	    	if (end_point2.chain_id > 0) {
	    		// The other side is part of the output path. Don't connect to end_point2, update end_point1 and try another one.
	    		valid = false;
	    	} else {
				// End points of the opposite ends of the segments.
				end_point1_other_chain_id = equivalent_chain(end_points[(&end_point1 - &end_points.front()) ^ 1].chain_id);
				end_point2_other_chain_id = equivalent_chain(end_points[(&end_point2 - &end_points.front()) ^ 1].chain_id);
				if (end_point1_other_chain_id == end_point2_other_chain_id && end_point1_other_chain_id != 0)
					// This edge forms a loop. Update end_point1 and try another one.
					valid = false;
	    	}
	    	if (valid) {
		    	// Remove the first and second point from the queue.
				queue.pop();
		    	queue.remove(end_point2.heap_idx);
		    	assert(end_point1.edge_out = &end_point2);
		    	end_point2.edge_out = &end_point1;
		    	end_point2.distance_out = end_point1.distance_out;
		    	// Assign chain IDs to the newly connected end points, set equivalent_chain if two chains were merged.
		    	size_t chain_id =
					(end_point1_other_chain_id == 0) ?
						((end_point2_other_chain_id == 0) ? equivalent_chain.next() : end_point2_other_chain_id) :
						((end_point2_other_chain_id == 0) ? end_point1_other_chain_id : 
							(end_point1_other_chain_id == end_point2_other_chain_id) ? 
								end_point1_other_chain_id :
								equivalent_chain.merge(end_point1_other_chain_id, end_point2_other_chain_id));
				end_point1.chain_id = chain_id;
				end_point2.chain_id = chain_id;
				assert(validate_graph_and_queue());
				if (iter == 0) {
					// Last iteration. There shall be exactly one or two end points waiting to be connected.
					assert(queue.size() == ((first_point == nullptr) ? 2 : 1));
					if (first_point == nullptr) {
						first_point = queue.top();
						queue.pop();
						first_point->edge_out = nullptr;
					}
					last_point = queue.top();
					last_point->edge_out = nullptr;
					queue.pop();
					assert(queue.empty());
					break;
				}
	    	} else {
				// This edge forms a loop. Update end_point1 and try another one.
				++ iter;
				end_point1.edge_out = nullptr;
		    	// Update edge_out and distance.
		    	size_t this_idx = &end_point1 - &end_points.front();
		    	// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the filter lambda).
				size_t next_idx = find_closest_point(kdtree, end_point1.pos, [&end_points, &equivalent_chain, this_idx](size_t idx) { 
			    	assert(end_points[this_idx].edge_out == nullptr);
			    	assert(end_points[this_idx].chain_id == 0);
					if ((idx ^ this_idx) <= 1 || end_points[idx].chain_id != 0)
						// Points of the same segment shall not be connected,
						// cannot connect to an already connected point (ideally those would be removed from the KD tree, but the update is difficult).
						return false;
			    	size_t chain1 = equivalent_chain(end_points[this_idx ^ 1].chain_id);
			    	size_t chain2 = equivalent_chain(end_points[idx      ^ 1].chain_id);
			    	return chain1 != chain2 || chain1 == 0;
				});
				assert(next_idx < end_points.size());
				end_point1.edge_out = &end_points[next_idx];
				end_point1.distance_out = (end_points[next_idx].pos - end_point1.pos).squaredNorm();
#ifndef NDEBUG
				// Each edge shall be longer than the last one removed from the queue.
				assert(end_point1.distance_out > distance_taken_last - SCALED_EPSILON);
#endif /* NDEBUG */
				// Update position of this end point in the queue based on the distance calculated at the line above.
				queue.update(end_point1.heap_idx);
		    	//FIXME Remove the other end point from the KD tree.
		    	// As the KD tree update is expensive, do it only after some larger number of points is removed from the queue.
				assert(validate_graph_and_queue());
	    	}
		}
		assert(queue.empty());

		// Now interconnect pairs of segments into a chain.
		assert(first_point != nullptr);
		out.reserve(num_segments);
		bool      failed = false;
		do {
			assert(out.size() < num_segments);
			size_t    		 first_point_id = first_point - &end_points.front();
			size_t           segment_id 	= first_point_id >> 1;
			bool             reverse        = (first_point_id & 1) != 0;
			EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
			if (REVERSE_COULD_FAIL) {
				if (reverse && ! could_reverse_func(segment_id)) {
					failed = true;
					break;
				}
			} else {
				assert(! reverse || could_reverse_func(segment_id));
			}
			out.emplace_back(segment_id, reverse);
			first_point = second_point->edge_out;
		} while (first_point != nullptr);
		if (REVERSE_COULD_FAIL) {
			if (failed) {
				if (start_near == nullptr) {
					// We may try the reverse order.
					out.clear();
					first_point = last_point;
					failed = false;
					do {
						assert(out.size() < num_segments);
						size_t    		 first_point_id = first_point - &end_points.front();
						size_t           segment_id 	= first_point_id >> 1;
						bool             reverse        = (first_point_id & 1) != 0;
						EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
						if (reverse && ! could_reverse_func(segment_id)) {
							failed = true;
							break;
						}
						out.emplace_back(segment_id, reverse);
						first_point = second_point->edge_out;
					} while (first_point != nullptr);
				}
			}
			if (failed)
				// As a last resort, try a dumb algorithm, which is not sensitive to edge reversal constraints.
				out = chain_segments_closest_point<EndPoint, decltype(kdtree), CouldReverseFunc>(end_points, kdtree, could_reverse_func, (initial_point != nullptr) ? *initial_point : end_points.front());
		} else {
			assert(! failed);
		}
	}

	assert(out.size() == num_segments);
	return out;
}

template<typename QueueType, typename KDTreeType, typename ChainsType, typename EndPointType>
void update_end_point_in_queue(QueueType &queue, const KDTreeType &kdtree, ChainsType &chains, std::vector<EndPointType> &end_points, EndPointType &end_point, size_t first_point_idx, const EndPointType *first_point)
{
	// Updating an end point or a 2nd from an end point.
	size_t this_idx = end_point.index(end_points);
	// If this segment is not the starting segment, then this end point or the opposite is unconnected.
	assert(first_point_idx == this_idx || first_point_idx == (this_idx ^ 1) || end_point.chain_id == 0 || end_point.opposite(end_points).chain_id == 0);
	end_point.edge_candidate = nullptr;
	if (first_point_idx == this_idx || (end_point.chain_id > 0 && first_point_idx == (this_idx ^ 1)))
	{
		// One may never flip the 1st edge, don't try it again.
		if (! end_point.heap_idx_invalid())
			queue.remove(end_point.heap_idx);
	}
	else
	{
		// Update edge_candidate and distance.
		size_t chain1a    = end_point.chain_id;
		size_t chain1b    = end_points[this_idx ^ 1].chain_id;
		size_t this_chain = chains.equivalent(std::max(chain1a, chain1b));
		// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the filter lambda).
		size_t next_idx = find_closest_point(kdtree, end_point.pos, [&end_points, &chains, this_idx, first_point_idx, first_point, this_chain](size_t idx) {
	    	assert(end_points[this_idx].edge_candidate == nullptr);
	    	// Either this end of the edge or the other end of the edge is not yet connected.
	    	assert((end_points[this_idx    ].chain_id == 0 && end_points[this_idx    ].edge_out == nullptr) ||
	    		   (end_points[this_idx ^ 1].chain_id == 0 && end_points[this_idx ^ 1].edge_out == nullptr));
			if ((idx ^ this_idx) <= 1 || idx == first_point_idx)
				// Points of the same segment shall not be connected.
				// Don't connect to the first point, we must not flip the 1st edge.
				return false;
			size_t chain2a = end_points[idx].chain_id;
			size_t chain2b = end_points[idx ^ 1].chain_id;
			if (chain2a > 0 && chain2b > 0)
				// Only unconnected end point or a point next to an unconnected end point may be connected to.
				// Ideally those would be removed from the KD tree, but the update is difficult.
				return false;
	    	assert(chain2a == 0 || chain2b == 0);
	    	size_t chain2 = chains.equivalent(std::max(chain2a, chain2b));
	    	if (this_chain == chain2)
	    		// Don't connect back to the same chain, don't create a loop.
	    		return this_chain == 0;
	    	// Don't connect to a segment requiring flipping if the segment starts or ends with the first point.
			if (chain2a > 0) {
				// Chain requires flipping.
				assert(chain2b == 0);
				auto &chain = chains.chain(chain2);
				if (chain.begin == first_point || chain.end == first_point)
					return false;
			}
			// Everything is all right, try to connect.
			return true;
		});
		assert(next_idx < end_points.size());
		assert(chains.equivalent(end_points[next_idx].chain_id) != chains.equivalent(end_points[next_idx ^ 1].chain_id) || end_points[next_idx].chain_id == 0);
		end_point.edge_candidate = &end_points[next_idx];
		end_point.distance_out   = (end_points[next_idx].pos - end_point.pos).norm();
		if (end_point.chain_id > 0)
			end_point.distance_out += chains.chain_flip_penalty(this_chain);
		if (end_points[next_idx].chain_id > 0)
			// The candidate chain is flipped.
			end_point.distance_out += chains.chain_flip_penalty(end_points[next_idx].chain_id);
		// Update position of this end point in the queue based on the distance calculated at the line above.
		if (end_point.heap_idx_invalid())
			queue.push(&end_point);
		else
			queue.update(end_point.heap_idx);
	}
}

template<typename PointType, typename SegmentEndPointFunc, bool REVERSE_COULD_FAIL, typename CouldReverseFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy_constrained_reversals2_(SegmentEndPointFunc end_point_func, CouldReverseFunc could_reverse_func, size_t num_segments, const PointType *start_near)
{
	std::vector<std::pair<size_t, bool>> out;

	if (num_segments == 0) {
		// Nothing to do.
	} 
	else if (num_segments == 1)
	{
		// Just sort the end points so that the first point visited is closest to start_near.
		out.emplace_back(0, start_near != nullptr && 
            (end_point_func(0, true) - *start_near).template cast<double>().squaredNorm() < (end_point_func(0, false) - *start_near).template cast<double>().squaredNorm());
	} 
	else
	{
		// End points of segments for the KD tree closest point search.
		// A single end point is inserted into the search structure for loops, two end points are entered for open paths.
		struct EndPoint {
			EndPoint(const Vec2d &pos) : pos(pos) {}
			Vec2d     pos;

			// Candidate for a new connection link.
			EndPoint *edge_candidate = nullptr;
			// Distance to the next end point following the link.
			// Zero value -> start of the final path.
			double    distance_out = std::numeric_limits<double>::max();

			size_t    heap_idx = std::numeric_limits<size_t>::max();
			bool 	  heap_idx_invalid() const { return this->heap_idx == std::numeric_limits<size_t>::max(); }

			// Identifier of the chain, to which this end point belongs. Zero means unassigned.
			size_t    chain_id = 0;
			// Double linked chain of segment end points in current path.
			EndPoint *edge_out = nullptr;

			size_t 				index(std::vector<EndPoint>          &endpoints) const { return this - endpoints.data(); }
			// Opposite end point of the same segment.
			EndPoint& 			opposite(std::vector<EndPoint>       &endpoints)       { return endpoints[(this - endpoints.data()) ^ 1]; }
			const EndPoint& 	opposite(const std::vector<EndPoint> &endpoints) const { return endpoints[(this - endpoints.data()) ^ 1]; }
		};

	    std::vector<EndPoint> end_points;
	    end_points.reserve(num_segments * 2);
	    for (size_t i = 0; i < num_segments; ++ i) {
            end_points.emplace_back(end_point_func(i, true ).template cast<double>());
            end_points.emplace_back(end_point_func(i, false).template cast<double>());
	    }

	    // Construct the closest point KD tree over end points of segments.
		auto coordinate_fn = [&end_points](size_t idx, size_t dimension) -> double { return end_points[idx].pos[dimension]; };
		KDTreeIndirect<2, double, decltype(coordinate_fn)> kdtree(coordinate_fn, end_points.size());

	    // Chained segments with their sum of connection lengths.
	    // The chain supports flipping all the segments, connecting the segments at the opposite ends.
	    // (this is a very useful path optimization for infill lines).
	    struct Chain {
	    	size_t		 num_segments	 = 0;
	    	double 		 cost 			 = 0.;
	    	double 		 cost_flipped	 = 0.;
	    	EndPoint 	*begin 			 = nullptr;
	    	EndPoint 	*end 			 = nullptr;
	    	size_t  	 equivalent_with = 0;

	    	// Flipping the chain has a time complexity of O(n).
	    	void flip(std::vector<EndPoint> &endpoints)
	    	{
				assert(this->num_segments > 1);
				assert(this->begin->edge_out == nullptr);
	    		assert(this->end  ->edge_out == nullptr);
				assert(this->begin->opposite(endpoints).edge_out != nullptr);
				assert(this->end  ->opposite(endpoints).edge_out != nullptr);
				// Start of the current segment processed.
	    		EndPoint *ept      = this->begin;
	    		// Previous end point to connect the other side of ept to.
	    		EndPoint *ept_prev = nullptr;
	    		do {
	    			EndPoint *ept_end      = &ept->opposite(endpoints);
	    			EndPoint *ept_next     = ept_end->edge_out;
	    			assert(ept_next == nullptr || ept_next->edge_out == ept_end);
	    			// Connect to the preceding segment.
	    			ept_end->edge_out = ept_prev;
	    			if (ept_prev != nullptr)
	    				ept_prev->edge_out = ept_end;
	    			ept_prev = ept;
	    			ept = ept_next;
	    		} while (ept != nullptr);
	    		ept_prev->edge_out = nullptr;
	    		// Swap the costs.
	    		std::swap(this->cost, this->cost_flipped);
				// Swap the ends.
				EndPoint *new_begin = &this->begin->opposite(endpoints);
				EndPoint *new_end   = &this->end->opposite(endpoints);
				std::swap(this->begin->chain_id, new_begin->chain_id);
				std::swap(this->end  ->chain_id, new_end  ->chain_id);
				this->begin = new_begin;
				this->end   = new_end;
				assert(this->begin->edge_out == nullptr);
	    		assert(this->end  ->edge_out == nullptr);
				assert(this->begin->opposite(endpoints).edge_out != nullptr);
				assert(this->end  ->opposite(endpoints).edge_out != nullptr);
			}

	    	double flip_penalty() const { return this->cost_flipped - this->cost; }
	    };

		// Helper to detect loops in already connected paths and to accomodate flipping of chains.
		//	
		// Unique chain IDs are assigned to paths. If paths are connected, end points will not have their chain IDs updated, but the chain IDs
		// will remember an "equivalent" chain ID, which is the lowest ID of all the IDs in the path, and the lowest ID is equivalent to itself.
		// Chain IDs are indexed starting with 1.
		// 
		// Chains remember their lengths and their lengths when each segment of the chain is flipped.
		class Chains {
		public:
			// Zero'th chain ID is invalid.
			Chains(size_t reserve) { 
				m_chains.reserve(reserve / 2);
				// Indexing starts with 1.
				m_chains.emplace_back();
			}

			// Generate next equivalence class.
			size_t 				next_id() {
				m_chains.emplace_back();
				m_chains.back().equivalent_with = ++ m_last_chain_id;
				return m_last_chain_id;
			}

			// Get equivalence class for chain ID, update the "equivalent_with" along the equivalence path.
			size_t 				equivalent(size_t chain_id) {
				if (chain_id != 0) {
					for (size_t last = chain_id;;) {
						size_t lower = m_chains[last].equivalent_with;
						if (lower == last) {
							m_chains[chain_id].equivalent_with = lower;
							chain_id = lower;
							break;
						}
						last = lower;
					}
				}
				return chain_id;
			}

			// Return a lowest chain ID of the two input chains.
			// Produce a new chain ID of both chain IDs are zero.
			size_t 			  	merge(size_t chain_id1, size_t chain_id2) {
				if (chain_id1 == 0)
					return (chain_id2 == 0) ? this->next_id() : chain_id2;
				if (chain_id2 == 0)
					return chain_id1;
				assert(m_chains[chain_id1].equivalent_with == chain_id1);
				assert(m_chains[chain_id2].equivalent_with == chain_id2);
				size_t chain_id = std::min(chain_id1, chain_id2);
				m_chains[chain_id1].equivalent_with = chain_id;
				m_chains[chain_id2].equivalent_with = chain_id;
				return chain_id;
			}

			Chain& 				chain(size_t chain_id) 		 { return m_chains[chain_id]; }
			const Chain&		chain(size_t chain_id) const { return m_chains[chain_id]; }

			double 				chain_flip_penalty(size_t chain_id) {
				chain_id = this->equivalent(chain_id);
				return m_chains[chain_id].flip_penalty();
			}

#ifndef NDEBUG
			bool				validate()
			{
				// Validate that the segments merged chain IDs make up a directed acyclic graph
				// with edges oriented towards the lower chain ID, therefore all ending up
				// in the lowest chain ID of all of them.
				assert(m_last_chain_id >= 0);
				assert(m_last_chain_id + 1 == m_chains.size());
				for (size_t i = 0; i < m_chains.size(); ++ i) {
					for (size_t last = i;;) {
						size_t lower = m_chains[last].equivalent_with;
						assert(lower <= last);
						if (lower == last)
							break;
						last = lower;
					}
				}
				return true;
			}
#endif /* NDEBUG */

		private:
			std::vector<Chain> 	m_chains;
			// Unique chain ID assigned to chains of end points of segments.
			size_t              m_last_chain_id = 0;
		} chains(num_segments);

		// Find the first end point closest to start_near.
		EndPoint *first_point = nullptr;
		size_t    first_point_idx = std::numeric_limits<size_t>::max();
		if (start_near != nullptr) {
            size_t idx = find_closest_point(kdtree, start_near->template cast<double>());
			assert(idx < end_points.size());
			first_point = &end_points[idx];
			first_point->distance_out = 0.;
			first_point->chain_id = chains.next_id();
			Chain &chain = chains.chain(first_point->chain_id);
			chain.begin = first_point;
			chain.end   = &first_point->opposite(end_points);
			first_point_idx = idx;
		}
		EndPoint *initial_point = first_point;
		EndPoint *last_point = nullptr;

		// Assign the closest point and distance to the end points.
		for (EndPoint &end_point : end_points) {
	    	assert(end_point.edge_candidate == nullptr);
	    	if (&end_point != first_point) {
		    	size_t this_idx = end_point.index(end_points);
		    	// Find the closest point to this end_point, which lies on a different extrusion path (filtered by the lambda).
		    	// Ignore the starting point as the starting point is considered to be occupied, no end point coud connect to it.
				size_t next_idx = find_closest_point(kdtree, end_point.pos, 
					[this_idx, first_point_idx](size_t idx){ return idx != first_point_idx && (idx ^ this_idx) > 1; });
				assert(next_idx < end_points.size());
				EndPoint &end_point2 = end_points[next_idx];
				end_point.edge_candidate = &end_point2;
				end_point.distance_out = (end_point2.pos - end_point.pos).norm();
			}
		}

	    // Initialize a heap of end points sorted by the lowest distance to the next valid point of a path.
	    auto queue = make_mutable_priority_queue<EndPoint*, true>(
			[](EndPoint *ep, size_t idx){ ep->heap_idx = idx; }, 
	    	[](EndPoint *l, EndPoint *r){ return l->distance_out < r->distance_out; });
		queue.reserve(end_points.size() * 2);
	    for (EndPoint &ep : end_points)
	    	if (first_point != &ep)
				queue.push(&ep);

#ifndef NDEBUG
		auto validate_graph_and_queue = [&chains, &end_points, &queue, first_point]() -> bool {
			assert(chains.validate());
			for (EndPoint &ep : end_points) {
				if (ep.heap_idx < queue.size()) {
					// End point is on the heap.
					assert(*(queue.cbegin() + ep.heap_idx) == &ep);
					// One side or the other of the segment is not yet connected.
					assert(ep.chain_id == 0 || ep.opposite(end_points).chain_id == 0);
				} else {
					// End point is NOT on the heap, therefore it must part of the output path.
					assert(ep.heap_idx_invalid());
					assert(ep.chain_id != 0);
					if (&ep == first_point) {
						assert(ep.edge_out == nullptr);
					} else {
						assert(ep.edge_out != nullptr);
						// Detect loops.
						for (EndPoint *pt = &ep; pt != nullptr;) {
							// Out of queue. It is a final point.
							EndPoint *pt_other = &pt->opposite(end_points);
							if (pt_other->heap_idx < queue.size()) {
								// The other side of this segment is undecided yet.
								// assert(pt_other->edge_out == nullptr);
								break;
							}
							pt = pt_other->edge_out;
						}
					}
				}
			}
			for (EndPoint *ep : queue)
				// Points in the queue or the opposites of the same segment are not connected yet.
				assert(ep->chain_id == 0 || ep->opposite(end_points).chain_id == 0);
			return true;
		};
#endif /* NDEBUG */

	    // Chain the end points: find (num_segments - 1) shortest links not forming bifurcations or loops.
		assert(num_segments >= 2);
#ifndef NDEBUG
		double distance_taken_last = 0.;
#endif /* NDEBUG */
		// Some links stored onto the priority queue are being invalidated during the calculation and they are not
		// updated immediately. If such a situation is detected for an end point pulled from the priority queue,
		// the end point is being updated and re-inserted into the priority queue. Therefore the number of iterations
		// required is higher than expected (it would be the number of links, num_segments - 1).
		// The limit here may not be necessary, but it guards us against an endless loop if something goes wrong.
		size_t num_iter = num_segments * 16;
		for (size_t num_connections_to_end = num_segments - 1; num_iter > 0; -- num_iter) {
			assert(validate_graph_and_queue());
	    	// Take the first end point, for which the link points to the currently closest valid neighbor.
	    	EndPoint *end_point1       = queue.top();
	    	assert(end_point1 != first_point);
	    	EndPoint *end_point1_other = &end_point1->opposite(end_points);
	    	// true if end_point1 is not the end of its chain, but the 2nd point. When connecting to the 2nd point, this chain needs
	    	// to be flipped first.
	    	bool      chain1_flip      = end_point1->chain_id > 0;
	    	// Either this point at the queue is not connected, or it is the 2nd point of a chain.
	    	// If connecting to a 2nd point of a chain, the 1st point shall not yet be connected and this chain will need
	    	// to be flipped.
	    	assert(  chain1_flip || (end_point1->chain_id == 0 && end_point1->edge_out == nullptr));
	    	assert(! chain1_flip || (end_point1_other->chain_id == 0 && end_point1_other->edge_out == nullptr));
			assert(end_point1->edge_candidate != nullptr);
#ifndef NDEBUG 
			// Each edge added shall be longer than the previous one taken.
			//assert(end_point1->distance_out > distance_taken_last - SCALED_EPSILON);
			if (end_point1->distance_out < distance_taken_last - SCALED_EPSILON) {
//				printf("Warning: taking shorter length than previously is suspicious\n");
			}
			distance_taken_last = end_point1->distance_out;
#endif /* NDEBUG */
	    	// Take the closest end point to the first end point,
	    	EndPoint *end_point2 	   = end_point1->edge_candidate;
	    	EndPoint *end_point2_other = &end_point2->opposite(end_points);
	    	bool      chain2_flip      = end_point2->chain_id > 0;
	    	// Is the link from end_point1 to end_point2 still valid? If yes, the link may be taken. Otherwise the link needs to be refreshed.
	    	bool      valid            = true;
	    	size_t 	  end_point1_chain_id = 0;
	    	size_t    end_point2_chain_id = 0;
	    	if (end_point2->chain_id > 0 && end_point2_other->chain_id > 0) {
	    		// The other side is part of the output path. Don't connect to end_point2, update end_point1 and try another one.
	    		valid = false;
	    	} else {
				// End points of the opposite ends of the segments.
				end_point1_chain_id = chains.equivalent((chain1_flip ? end_point1 : end_point1_other)->chain_id);
				end_point2_chain_id = chains.equivalent((chain2_flip ? end_point2 : end_point2_other)->chain_id);
				if (end_point1_chain_id == end_point2_chain_id && end_point1_chain_id != 0)
					// This edge forms a loop. Update end_point1 and try another one.
					valid = false;
				else {
					// Verify whether end_point1.distance_out still matches the current state of the two end points to be connected and their chains.
					// Namely, the other chain may have been flipped in the meantime.
					double dist = (end_point2->pos - end_point1->pos).norm();
					if (chain1_flip)
						dist += chains.chain_flip_penalty(end_point1_chain_id);
					if (chain2_flip)
						dist += chains.chain_flip_penalty(end_point2_chain_id);
					if (std::abs(dist - end_point1->distance_out) > SCALED_EPSILON)
						// The distance changed due to flipping of one of the chains. Refresh this end point in the queue.
						valid = false;
				}
				if (valid && first_point != nullptr) {
					// Verify that a chain starting or ending with the first_point does not get flipped.
					if (chain1_flip) {
						Chain &chain = chains.chain(end_point1_chain_id);
						if (chain.begin == first_point || chain.end == first_point)
							valid = false;
					}
					if (valid && chain2_flip) {
						Chain &chain = chains.chain(end_point2_chain_id);
						if (chain.begin == first_point || chain.end == first_point)
							valid = false;
					}
				}
	    	}
	    	if (valid) {
		    	// Remove the first and second point from the queue.
				queue.pop();
		    	queue.remove(end_point2->heap_idx);
		    	assert(end_point1->edge_candidate == end_point2);
		    	end_point1->edge_candidate = nullptr;
		    	Chain *chain1 = (end_point1_chain_id == 0) ? nullptr : &chains.chain(end_point1_chain_id);
		    	Chain *chain2 = (end_point2_chain_id == 0) ? nullptr : &chains.chain(end_point2_chain_id);
				assert(chain1 == nullptr || (chain1_flip ? (chain1->begin == end_point1_other || chain1->end == end_point1_other) : (chain1->begin == end_point1 || chain1->end == end_point1)));
				assert(chain2 == nullptr || (chain2_flip ? (chain2->begin == end_point2_other || chain2->end == end_point2_other) : (chain2->begin == end_point2 || chain2->end == end_point2)));
				if (chain1_flip)
		    		chain1->flip(end_points);
		    	if (chain2_flip)
		    		chain2->flip(end_points);
				assert(chain1 == nullptr || chain1->begin == end_point1 || chain1->end == end_point1);
				assert(chain2 == nullptr || chain2->begin == end_point2 || chain2->end == end_point2);
		    	size_t chain_id = chains.merge(end_point1_chain_id, end_point2_chain_id);
				Chain &chain    = chains.chain(chain_id);
				{
				    Chain chain_dst;
					chain_dst.begin = (chain1 == nullptr) ? end_point1_other : (chain1->begin == end_point1) ? chain1->end : chain1->begin;
			    	chain_dst.end   = (chain2 == nullptr) ? end_point2_other : (chain2->begin == end_point2) ? chain2->end : chain2->begin;
			    	chain_dst.cost  = (chain1 == 0 ? 0. : chain1->cost) + (chain2 == 0 ? 0. : chain2->cost) + (end_point2->pos - end_point1->pos).norm();
			    	chain_dst.cost_flipped = (chain1 == 0 ? 0. : chain1->cost_flipped) + (chain2 == 0 ? 0. : chain2->cost_flipped) + (end_point2_other->pos - end_point1_other->pos).norm();
			    	chain_dst.num_segments = (chain1 == 0 ? 1 : chain1->num_segments) + (chain2 == 0 ? 1 : chain2->num_segments);
				    chain_dst.equivalent_with = chain_id;
					chain = chain_dst;
			    }
				if (chain.begin != end_point1_other && ! end_point1_other->heap_idx_invalid())
					queue.remove(end_point1_other->heap_idx);
				if (chain.end != end_point2_other && ! end_point2_other->heap_idx_invalid())
					queue.remove(end_point2_other->heap_idx);
				end_point1->edge_out = end_point2;
				end_point2->edge_out = end_point1;
				end_point1->chain_id = chain_id;
				end_point2->chain_id = chain_id;
				end_point1_other->chain_id = chain_id;
				end_point2_other->chain_id = chain_id;
				if (chain.begin != first_point)
					chain.begin->chain_id = 0;
				if (chain.end != first_point)
					chain.end->chain_id = 0;
				if (-- num_connections_to_end == 0) {
					assert(validate_graph_and_queue());
					// Last iteration. There shall be exactly one or two end points waiting to be connected.
					assert(queue.size() <= ((first_point == nullptr) ? 4 : 2));
					if (first_point == nullptr) {
						// Find the first remaining end point.
						do {
							first_point = queue.top();
							queue.pop();
						} while (first_point->edge_out != nullptr);
						assert(first_point->edge_out == nullptr);
					}
					// Find the first remaining end point.
					do {
						last_point = queue.top();
						queue.pop();
					} while (last_point->edge_out != nullptr);
					assert(last_point->edge_out == nullptr);
#ifndef NDEBUG
					while (! queue.empty()) {
						assert(queue.top()->edge_out != nullptr && queue.top()->chain_id > 0);
						queue.pop();
					}
#endif /* NDEBUG */
					break;
				} else {
					//FIXME update the 2nd end points on the queue.
					// Update end points of the flipped segments.
					update_end_point_in_queue(queue, kdtree, chains, end_points, chain.begin->opposite(end_points), first_point_idx, first_point);
					update_end_point_in_queue(queue, kdtree, chains, end_points, chain.end->opposite(end_points),   first_point_idx, first_point);
					if (chain1_flip)
						update_end_point_in_queue(queue, kdtree, chains, end_points, *chain.begin, first_point_idx, first_point);
					if (chain2_flip)
						update_end_point_in_queue(queue, kdtree, chains, end_points, *chain.end,   first_point_idx, first_point);
					// End points of chains shall certainly stay in the queue.
					assert(chain.begin == first_point || chain.begin->heap_idx < queue.size());
					assert(chain.end   == first_point || chain.end  ->heap_idx < queue.size());
					assert(&chain.begin->opposite(end_points) != first_point &&
						(chain.begin == first_point ? chain.begin->opposite(end_points).heap_idx_invalid() : chain.begin->opposite(end_points).heap_idx < queue.size()));
					assert(&chain.end  ->opposite(end_points) != first_point &&
						(chain.end   == first_point ? chain.end  ->opposite(end_points).heap_idx_invalid() : chain.end  ->opposite(end_points).heap_idx < queue.size()));

				}
			} else {
				// This edge forms a loop. Update end_point1 and try another one.
				update_end_point_in_queue(queue, kdtree, chains, end_points, *end_point1, first_point_idx, first_point);
#ifndef NDEBUG
				// Each edge shall be longer than the last one removed from the queue.
				//assert(end_point1->distance_out > distance_taken_last - SCALED_EPSILON);
				if (end_point1->distance_out < distance_taken_last - SCALED_EPSILON) {
//					printf("Warning: taking shorter length than previously is suspicious\n");
				}
#endif /* NDEBUG */
		    	//FIXME Remove the other end point from the KD tree.
		    	// As the KD tree update is expensive, do it only after some larger number of points is removed from the queue.
		    }
			assert(validate_graph_and_queue());
		}
		assert(queue.empty());

		// Now interconnect pairs of segments into a chain.
		assert(first_point != nullptr);
		out.reserve(num_segments);
		bool      failed = false;
		do {
			assert(out.size() < num_segments);
			size_t    		 first_point_id = first_point - &end_points.front();
			size_t           segment_id 	= first_point_id >> 1;
			bool             reverse        = (first_point_id & 1) != 0;
			EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
			if (REVERSE_COULD_FAIL) {
				if (reverse && ! could_reverse_func(segment_id)) {
					failed = true;
					break;
				}
			} else {
				assert(! reverse || could_reverse_func(segment_id));
			}
			out.emplace_back(segment_id, reverse);
			first_point = second_point->edge_out;
		} while (first_point != nullptr);
		if (REVERSE_COULD_FAIL) {
			if (failed) {
				if (start_near == nullptr) {
					// We may try the reverse order.
					out.clear();
					first_point = last_point;
					failed = false;
					do {
						assert(out.size() < num_segments);
						size_t    		 first_point_id = first_point - &end_points.front();
						size_t           segment_id 	= first_point_id >> 1;
						bool             reverse        = (first_point_id & 1) != 0;
						EndPoint 		*second_point   = &end_points[first_point_id ^ 1];
						if (reverse && ! could_reverse_func(segment_id)) {
							failed = true;
							break;
						}
						out.emplace_back(segment_id, reverse);
						first_point = second_point->edge_out;
					} while (first_point != nullptr);
				}
			}
			if (failed)
				// As a last resort, try a dumb algorithm, which is not sensitive to edge reversal constraints.
				out = chain_segments_closest_point<EndPoint, decltype(kdtree), CouldReverseFunc>(end_points, kdtree, could_reverse_func, (initial_point != nullptr) ? *initial_point : end_points.front());
		} else {
			assert(! failed);
		}
	}

	assert(out.size() == num_segments);
	return out;
}

template<typename PointType, typename SegmentEndPointFunc, typename CouldReverseFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy_constrained_reversals(SegmentEndPointFunc end_point_func, CouldReverseFunc could_reverse_func, size_t num_segments, const PointType *start_near)
{
	return chain_segments_greedy_constrained_reversals_<PointType, SegmentEndPointFunc, true, CouldReverseFunc>(end_point_func, could_reverse_func, num_segments, start_near);
}

template<typename PointType, typename SegmentEndPointFunc>
std::vector<std::pair<size_t, bool>> chain_segments_greedy(SegmentEndPointFunc end_point_func, size_t num_segments, const PointType *start_near)
{
	auto could_reverse_func = [](size_t /* idx */) -> bool { return true; };
	return chain_segments_greedy_constrained_reversals_<PointType, SegmentEndPointFunc, false, decltype(could_reverse_func)>(end_point_func, could_reverse_func, num_segments, start_near);
}

} // namespace vendored
static std::vector<size_t> run_chain_points(const std::vector<Vec2d> &pts) {
    auto segment_end_point = [&pts](size_t idx, bool) -> const Vec2d & { return pts[idx]; };
    auto ordered = vendored::chain_segments_greedy<Vec2d, decltype(segment_end_point)>(segment_end_point, pts.size(), (const Vec2d *)nullptr);
    std::vector<size_t> out;
    for (auto &p : ordered) out.push_back(p.first);
    return out;
}
int main() {
    std::vector<Vec2d> pts;
    double x, y;
    while (std::cin >> x >> y) pts.emplace_back(x, y);
    for (size_t i : run_chain_points(pts)) std::cout << i << " ";
    std::cout << std::endl;
    return 0;
}
