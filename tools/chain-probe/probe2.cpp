// Probe 2: run upstream chain_segments_greedy_constrained_reversals (v1) on an
// ARES_DUMP_CHAIN block (entities + seed) and print the resulting order.
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
    Vec2d() = default;
    Vec2d(double px, double py) : x(px), y(py) {}
    double operator[](size_t i) const { return i == 0 ? x : y; }
    template <typename T> Vec2d cast() const { return *this; }
    double squaredNorm() const { return x * x + y * y; }
};
static inline Vec2d operator-(const Vec2d &a, const Vec2d &b) { return Vec2d(a.x - b.x, a.y - b.y); }
static inline bool operator==(const Vec2d &a, const Vec2d &b) { return a.x == b.x && a.y == b.y; }
using Point = Vec2d;
#define SCALED_EPSILON 1.0e-4
static void svg_draw_polyline_chain(const char *, int, const Vec2d &) {}

#include "greedy1.inc"

static Vec2d parse_pair(const std::string &s) {
    size_t c = s.find(',');
    return Vec2d(std::stod(s.substr(0, c)), std::stod(s.substr(c + 1)));
}

int main(int argc, char **argv) {
    std::ifstream in(argv[1]);
    long want_block = argc > 2 ? std::stol(argv[2]) : 0; // 0 = last
    std::string line;
    long block = 0;
    Vec2d seed(0., 0.);
    std::vector<std::pair<Vec2d, Vec2d>> entities;
    std::vector<long> expected;
    std::vector<std::pair<Vec2d, Vec2d>> all_entities;
    std::vector<long> cur_expected;
    Vec2d cur_seed(0., 0.);
    while (std::getline(in, line)) {
        if (line.compare(0, 2, "C ") == 0) {
            if (block > 0 && (want_block == 0 || want_block == block)) {
                entities = all_entities; expected = cur_expected; seed = cur_seed;
            }
            block++;
            all_entities.clear(); cur_expected.clear();
            size_t p = line.find('(');
            size_t q = line.find(')', p);
            cur_seed = parse_pair(line.substr(p + 1, q - p - 1));
        } else if (line.compare(0, 2, "E ") == 0) {
            size_t f = line.find("f=(");
            size_t fe = line.find(')', f);
            Vec2d first = parse_pair(line.substr(f + 3, fe - f - 3));
            size_t l = line.find("l=(");
            size_t le = line.find(')', l);
            Vec2d last = parse_pair(line.substr(l + 3, le - l - 3));
            all_entities.emplace_back(first, last);
        } else if (line.compare(0, 2, "O ") == 0) {
            cur_expected.push_back(std::stol(line.substr(2)));
        }
    }
    if (block > 0 && (want_block == 0 || want_block == block)) {
        entities = all_entities; expected = cur_expected; seed = cur_seed;
    }
    std::cout << "entities: " << entities.size() << std::endl;
    auto end_point = [&entities](size_t idx, bool first) -> Point {
        return first ? entities[idx].first : entities[idx].second;
    };
    auto could_reverse = [](size_t) -> bool { return true; };
    std::vector<std::pair<size_t, bool>> out = chain_segments_greedy_constrained_reversals<Point, decltype(end_point), decltype(could_reverse)>(end_point, could_reverse, entities.size(), &seed);
    for (size_t i = 0; i < out.size(); ++i) {
        long exp = i < expected.size() ? expected[i] : -1;
        std::string mark = ((long)out[i].first == exp) ? std::string() : (std::string("  DIFF(exp=") + std::to_string(exp) + ")");
        std::printf("O %zu rev=%d%s\n", out[i].first, out[i].second, mark.c_str());
    }
    return 0;
}
