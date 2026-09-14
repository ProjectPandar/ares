// Stub for the chain-probe: std::allocator-compatible drop-in.
#include <cstddef>
#include <memory>
namespace tbb {
template <typename T>
using scalable_allocator = std::allocator<T>;
}
