// Test-only original glu-libtess oracle. Wrapper submission/expansion ports
// libslic3r/Tesselate.cpp:23-56,110-164,192-195; kernel is compiled unchanged.
#include <glu-libtess.h>
#include <array>
#include <cstdint>
#include <cstring>
#include <deque>
#include <iomanip>
#include <iostream>
#include <vector>

struct Vertex { std::array<double, 3> point; int id; };
struct Wrapper {
    GLUtesselator *tess = gluNewTess();
    std::deque<Vertex> vertices;
    std::vector<Vertex> output;
    std::vector<unsigned> primitives;
    std::vector<unsigned> errors;
    unsigned primitive = 0;
    int count = 0;
    Vertex first{}, second{};
    double z;
    bool flipped;
    Wrapper(double height, bool flip) : z(height), flipped(flip) {
        gluTessCallback(tess, GLU_TESS_BEGIN_DATA, (_GLUfuncptr)begin);
        gluTessCallback(tess, GLU_TESS_END_DATA, (_GLUfuncptr)end);
        gluTessCallback(tess, GLU_TESS_VERTEX_DATA, (_GLUfuncptr)vertex);
        gluTessCallback(tess, GLU_TESS_COMBINE_DATA, (_GLUfuncptr)combine);
        gluTessCallback(tess, GLU_TESS_ERROR_DATA, (_GLUfuncptr)error);
    }
    ~Wrapper() { gluDeleteTess(tess); }
    static void begin(unsigned which, void *data) {
        auto &self = *(Wrapper *)data;
        self.primitive = which; self.count = 0; self.primitives.push_back(which);
    }
    static void end(void *data) { ((Wrapper *)data)->count = 0; }
    static void error(unsigned code, void *data) { ((Wrapper *)data)->errors.push_back(code); }
    static void combine(double point[3], void *[4], float[4], void **out, void *data) {
        auto &self = *(Wrapper *)data;
        self.vertices.push_back({{point[0], point[1], self.z}, (int)self.vertices.size()});
        *out = &self.vertices.back();
    }
    static void vertex(void *data, void *object) {
        if (data == nullptr) return; // Tesselate.cpp:123-124.
        auto &self = *(Wrapper *)object;
        Vertex current = *(Vertex *)data;
        current.point[2] = self.z;
        ++self.count;
        if (self.count == 1) self.first = current;
        else if (self.count == 2) self.second = current;
        else {
            bool flip = self.flipped;
            if (self.primitive == GL_TRIANGLE_STRIP && self.count == 4) {
                flip = !flip; self.count = 2;
            }
            self.output.push_back(self.first);
            self.output.push_back(flip ? current : self.second);
            self.output.push_back(flip ? self.second : current);
            if (self.primitive == GL_TRIANGLE_STRIP) {
                self.first = self.second; self.second = current;
            } else if (self.primitive == GL_TRIANGLE_FAN) self.second = current;
            else self.count = 0;
        }
    }
};
int main() {
    double z; int flip, contour_count;
    std::cin >> z >> flip >> contour_count;
    Wrapper wrapper(z, flip != 0);
    gluTessBeginPolygon(wrapper.tess, &wrapper);
    for (int c = 0; c < contour_count; ++c) {
        int size; std::cin >> size;
        gluTessBeginContour(wrapper.tess);
        for (int i = 0; i < size; ++i) {
            long long x, y; std::cin >> x >> y;
            wrapper.vertices.push_back({{x * 0.000001, y * 0.000001, 0.0}, (int)wrapper.vertices.size()});
            auto &v = wrapper.vertices.back();
            gluTessVertex(wrapper.tess, v.point.data(), &v);
        }
        gluTessEndContour(wrapper.tess);
    }
    gluTessEndPolygon(wrapper.tess);
    std::cout << std::setprecision(17) << "{\"primitives\":[";
    for (size_t i = 0; i < wrapper.primitives.size(); ++i) {
        if (i) std::cout << ','; std::cout << wrapper.primitives[i];
    }
    std::cout << "],\"errors\":[";
    for (size_t i = 0; i < wrapper.errors.size(); ++i) {
        if (i) std::cout << ','; std::cout << wrapper.errors[i];
    }
    std::cout << "],\"vertices\":[";
    for (size_t i = 0; i < wrapper.vertices.size(); ++i) {
        if (i) std::cout << ',';
        auto &v = wrapper.vertices[i]; std::cout << '[' << v.point[0] << ',' << v.point[1] << ']';
    }
    std::cout << "],\"order\":[";
    for (size_t i = 0; i < wrapper.output.size(); ++i) {
        if (i) std::cout << ','; std::cout << wrapper.output[i].id;
    }
    std::cout << "],\"triangles\":[";
    for (size_t i = 0; i < wrapper.output.size(); ++i) {
        if (i) std::cout << ',';
        auto &v = wrapper.output[i]; std::cout << '[' << v.point[0] << ',' << v.point[1] << ',' << v.point[2] << ']';
    }
    std::cout << "],\"triangle_bits\":[";
    for (size_t i = 0; i < wrapper.output.size(); ++i) {
        if (i) std::cout << ',';
        std::cout << '[';
        for (int axis = 0; axis < 3; ++axis) {
            if (axis) std::cout << ',';
            std::uint64_t bits;
            std::memcpy(&bits, &wrapper.output[i].point[axis], sizeof(bits));
            std::cout << bits;
        }
        std::cout << ']';
    }
    std::cout << "]}\n";
    return wrapper.errors.empty() ? 0 : 1;
}
