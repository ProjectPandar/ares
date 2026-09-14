#include "clipper/clipper.hpp"
#include <cstdio>
#include <vector>
int main() {
    ClipperLib::Path subject = {{3783543,3783543},{-3783539,3783543},{-3783539,-3783539},{3783543,-3783539}};
    ClipperLib::ClipperOffset co(3.0);
    co.AddPath(subject, ClipperLib::jtMiter, ClipperLib::etClosedPolygon);
    ClipperLib::Paths sol;
    co.Execute(sol, 269112.96875);
    for (auto &p : sol) {
        std::printf("V n=%zu", p.size());
        for (auto &pt : p) std::printf(" (%lld,%lld)", (long long)pt.x(), (long long)pt.y());
        std::printf("\n");
    }
    return 0;
}
