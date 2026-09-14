// Standalone probe: run upstream chain_polylines (greedy2 + two-exchange) on the
// octagram fragments from the ARES_DUMP_PLANECLIP dump and print the chain order.
#include <cassert>
#include <cmath>
#define EPSILON 1.0e-4
#include "KDTreeIndirect.hpp"
#include "MutablePriorityQueue.hpp"

#include <algorithm>
#include <cstdio>
#include <fstream>
#include <iostream>
#include <limits>
#include <sstream>
#include <string>
#include <utility>
#include <vector>

struct Vec2d {
    double x = 0., y = 0.;
    double operator[](size_t i) const { return i == 0 ? x : y; }
    Vec2d() = default;
    Vec2d(double px, double py) : x(px), y(py) {}
    double norm() const { return std::sqrt(x * x + y * y); }
    double squaredNorm() const { return x * x + y * y; }
    template <typename T>
    Vec2d cast() const { return *this; }
};
static inline Vec2d operator-(const Vec2d &a, const Vec2d &b) { return Vec2d(a.x - b.x, a.y - b.y); }
static inline Vec2d operator+(const Vec2d &a, const Vec2d &b) { return Vec2d(a.x + b.x, a.y + b.y); }
static inline bool operator==(const Vec2d &a, const Vec2d &b) { return a.x == b.x && a.y == b.y; }
using Point = Vec2d;

struct Polyline {
    std::vector<Vec2d> points;
    bool empty() const { return points.empty(); }
    size_t size() const { return points.size(); }
    const Vec2d &first_point() const { return points.front(); }
    const Vec2d &last_point() const { return points.back(); }
    void reverse() { std::reverse(points.begin(), points.end()); }
    double length() const {
        double l = 0.;
        for (size_t i = 1; i < points.size(); ++i)
            l += (points[i] - points[i - 1]).norm();
        return l;
    }
};
using Polylines = std::vector<Polyline>;

#define SCALED_EPSILON 1.0e-4
static void svg_draw_polyline_chain(const char *, int, const Polylines &) {}

#include "flipedge.inc"
#include "mincross1.inc"
#include "mincross2.inc"
#include "mincross3.inc"
#include "reorder2.inc"
#include "improve2.inc"
#include "updateep.inc"
#include "greedy1.inc"
#include "greedy2.inc"
#include "chainpoly.inc"

int main(int argc, char **argv) {
    const char *path = argc > 1 ? argv[1] : "/tmp/oct-clip2.txt";
    std::ifstream in(path);
    std::string line;
    Polylines polylines;
    while (std::getline(in, line)) {
        size_t rpos = line.find("  R ");
        if (line.compare(0, 2, "P ") != 0 || rpos == std::string::npos)
            continue;
        std::string raw = line.substr(rpos + 3);
        Polyline pl;
        size_t pos = 0;
        std::vector<long long> nums;
        while ((pos = raw.find('(', pos)) != std::string::npos) {
            size_t end = raw.find(')', pos);
            std::string pair = raw.substr(pos + 1, end - pos - 1);
            size_t comma = pair.find(',');
            nums.push_back(std::stoll(pair.substr(0, comma)));
            nums.push_back(std::stoll(pair.substr(comma + 1)));
            pos = end;
        }
        for (size_t i = 0; i + 1 < nums.size(); i += 2)
            pl.points.emplace_back(double(nums[i]), double(nums[i + 1]));
        polylines.push_back(pl);
    }
    std::cout << "fragments: " << polylines.size() << std::endl;
    Polylines reference = polylines;
    Polylines chained = chain_polylines(std::move(polylines), nullptr);
    for (size_t i = 0; i < chained.size(); ++i) {
        size_t idx = std::numeric_limits<size_t>::max();
        bool flipped = false;
        for (size_t j = 0; j < reference.size(); ++j) {
            if (reference[j].size() == chained[i].size()
                && reference[j].first_point() == chained[i].first_point()
                && reference[j].last_point() == chained[i].last_point()) {
                idx = j;
                break;
            }
            if (reference[j].size() == chained[i].size()
                && reference[j].first_point() == chained[i].last_point()
                && reference[j].last_point() == chained[i].first_point()) {
                idx = j;
                flipped = true;
                break;
            }
        }
        std::printf("%zu: idx=%zu%s front=(%.3f,%.3f) back=(%.3f,%.3f)\n", i, idx,
                    flipped ? "R" : "",
                    chained[i].first_point().x, chained[i].first_point().y,
                    chained[i].last_point().x, chained[i].last_point().y);
    }
    return 0;
}
