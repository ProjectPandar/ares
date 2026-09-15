// probe8: vendored FillAdaptive octree + line generation (probe8 harness).
#include "p8shim.hpp"
#include <cassert>

namespace Slic3r { namespace FillAdaptive {

template<typename Vector>
bool triangle_AABB_intersects(const Vector &a, const Vector &b, const Vector &c, const BoundingBoxf3 &aabb)
{
    Vector tMin = a.cwiseMin(b.cwiseMin(c));
    Vector tMax = a.cwiseMax(b.cwiseMax(c));

    if (tMin.x() >= aabb.max.x() || tMax.x() <= aabb.min.x()
        || tMin.y() >= aabb.max.y() || tMax.y() <= aabb.min.y()
        || tMin.z() >= aabb.max.z() || tMax.z() <= aabb.min.z())
        return false;

    Vector center = (aabb.min + aabb.max) * 0.5;
    Vector h = aabb.max - center;

    const Vector t[3] { b-a, c-a, c-b };
    Vector ac = a - center;
    Vector n = t[0].cross(t[1]);
    double s = n.dot(ac);
    double r = std::abs(h.dot(n.cwiseAbs()));
    if (std::abs(s) >= r)
        return false;

    const Vector at[3] = { t[0].cwiseAbs(), t[1].cwiseAbs(), t[2].cwiseAbs() };
    Vector bc = b - center;
    Vector cc = c - center;

#define SAT_TEST(AXIS, A1, A2) \
    { \
        double d1 = t[AXIS].y() * A1.z() - t[AXIS].z() * A1.y(); \
        double d2 = t[AXIS].y() * A2.z() - t[AXIS].z() * A2.y(); \
        double tc = (d1 + d2) * 0.5; \
        r = std::abs(h.y() * at[AXIS].z() + h.z() * at[AXIS].y()); \
        if (r + std::abs(tc - d1) < std::abs(tc)) return false; \
    }
    SAT_TEST(0, ac, cc)
    SAT_TEST(1, ac, bc)
    SAT_TEST(2, ac, bc)
#undef SAT_TEST
#define SAT_TEST(AXIS, A1, A2) \
    { \
        double d1 = t[AXIS].z() * A1.x() - t[AXIS].x() * A1.z(); \
        double d2 = t[AXIS].z() * A2.x() - t[AXIS].x() * A2.z(); \
        double tc = (d1 + d2) * 0.5; \
        r = std::abs(h.x() * at[AXIS].z() + h.z() * at[AXIS].x()); \
        if (r + std::abs(tc - d1) < std::abs(tc)) return false; \
    }
    SAT_TEST(0, ac, cc)
    SAT_TEST(1, ac, bc)
    SAT_TEST(2, ac, bc)
#undef SAT_TEST
#define SAT_TEST(AXIS, A1, A2) \
    { \
        double d1 = t[AXIS].x() * A1.y() - t[AXIS].y() * A1.x(); \
        double d2 = t[AXIS].x() * A2.y() - t[AXIS].y() * A2.x(); \
        double tc = (d1 + d2) * 0.5; \
        r = std::abs(h.y() * at[AXIS].x() + h.x() * at[AXIS].y()); \
        if (r + std::abs(tc - d1) < std::abs(tc)) return false; \
    }
    SAT_TEST(0, ac, cc)
    SAT_TEST(1, ac, bc)
    SAT_TEST(2, ac, bc)
#undef SAT_TEST
    return true;
}

// Explicit instantiation for the shim vector type.
template bool triangle_AABB_intersects<Vec3d>(const Vec3d&, const Vec3d&, const Vec3d&, const BoundingBoxf3&);

}} // namespace Slic3r::FillAdaptive

