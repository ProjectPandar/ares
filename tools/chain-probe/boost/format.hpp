// Stub for the chain-probe (only used by Semver string formatting paths we
// never execute).
#include <sstream>
namespace boost {
struct format {
    std::ostringstream out;
    template <typename T> format &operator%(const T &v) { out << v; return *this; }
    std::string str() const { return out.str(); }
};
}
