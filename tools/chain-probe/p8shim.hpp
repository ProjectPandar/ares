#ifndef probe8_filladaptive_shim
#define probe8_filladaptive_shim
#include <algorithm>
#include <array>
#include <cassert>
#include <cmath>
#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <limits>
#include <memory>
#include <string>
#include <vector>

using coord_t = int64_t;
using coordf_t = double;
static constexpr double SCALED_EPSILON = 0.0000001;
#ifndef EPSILON
#define EPSILON 1e-4
#endif

struct Vec2d { double x_=0,y_=0; Vec2d()=default; Vec2d(double a,double b):x_(a),y_(b){} double x() const {return x_;} double y() const {return y_;} double& x(){return x_;} double& y(){return y_;}
  Vec2d operator+(const Vec2d&o) const { return Vec2d(x_+o.x_, y_+o.y_); }
  Vec2d operator-(const Vec2d&o) const { return Vec2d(x_-o.x_, y_-o.y_); }
  Vec2d operator*(double s) const { return Vec2d(x_*s, y_*s); }
  Vec2d& operator+=(const Vec2d&o) { x_+=o.x_; y_+=o.y_; return *this; }
  Vec2d& operator-=(const Vec2d&o) { x_-=o.x_; y_-=o.y_; return *this; } };
struct Vec3d { double v[3]{0,0,0}; Vec3d()=default; Vec3d(double a,double b,double c){v[0]=a;v[1]=b;v[2]=c;} double x() const {return v[0];} double y() const {return v[1];} double z() const {return v[2];} double& operator[](int i){return v[i];} double operator[](int i) const {return v[i];}
  Vec3d operator+(const Vec3d&o) const { return Vec3d(v[0]+o.v[0],v[1]+o.v[1],v[2]+o.v[2]); }
  Vec3d operator-(const Vec3d&o) const { return Vec3d(v[0]-o.v[0],v[1]-o.v[1],v[2]-o.v[2]); }
  Vec3d operator*(double s) const { return Vec3d(v[0]*s,v[1]*s,v[2]*s); }
  Vec3d cross(const Vec3d&o) const { return Vec3d(y()*o.z()-z()*o.y(), z()*o.x()-x()*o.z(), x()*o.y()-y()*o.x()); }
  double dot(const Vec3d&o) const { return x()*o.x()+y()*o.y()+z()*o.z(); }
  double norm() const { return std::sqrt(dot(*this)); }
  Vec3d cwiseMin(const Vec3d&o) const { return Vec3d(std::min(x(),o.x()),std::min(y(),o.y()),std::min(z(),o.z())); }
  Vec3d cwiseMax(const Vec3d&o) const { return Vec3d(std::max(x(),o.x()),std::max(y(),o.y()),std::max(z(),o.z())); }
  Vec3d cwiseAbs() const { return Vec3d(std::abs(x()),std::abs(y()),std::abs(z())); } };
static inline Vec3d operator*(double s, const Vec3d&o){ return o*s; }
using Vec3f = Vec3d;

struct Point { coord_t x_=0, y_=0; Point()=default; Point(coord_t x, coord_t y):x_(x),y_(y){} coord_t x() const {return x_;} coord_t y() const {return y_;}
  static Point new_scale(const Vec2d &v){ return Point(coord_t(std::round(v.x()*40.0)), coord_t(std::round(v.y()*40.0))); } };
struct Vec2crd { coord_t x_, y_; Vec2crd(coord_t a, coord_t b):x_(a),y_(b){} coord_t x() const {return x_;} coord_t y() const {return y_;} Vec2crd cwiseAbs() const { return Vec2crd(std::abs(x_), std::abs(y_)); } coord_t maxCoeff() const { return std::max(x_, y_); } };
static inline Vec2crd operator-(const Point&a, const Point&b) { return Vec2crd(a.x()-b.x(), a.y()-b.y()); }
struct Line { Point a, b; Line()=default; Line(const Point&A, const Point&B):a(A),b(B){} };

template<typename V> struct BoundingBox3Base { V min, max; BoundingBox3Base()=default; BoundingBox3Base(const V&a,const V&b):min(a),max(b){} V size() const { return max-min; } V center() const { return (min+max)*0.5; } };
using BoundingBoxf3 = BoundingBox3Base<Vec3d>;

inline bool shim_contains(const std::string &s, const std::string &n){ return s.find(n)!=std::string::npos; }
template <typename T> struct object_pool {
    ~object_pool() { for (T *p : items) delete p; }
    template <typename... A> T *construct(A&&... a) { T *p = new T(std::forward<A>(a)...); items.push_back(p); return p; }
    std::vector<T*> items;
};

namespace Slic3r { namespace FillAdaptive {
struct Cube;
struct Octree;
struct OctreeDeleter { void operator()(Octree *p); };
using OctreePtr = std::unique_ptr<Octree, OctreeDeleter>;
struct P8Mesh {
    std::vector<Vec3d> vertices;
    std::vector<std::array<int,3>> indices;
};
struct P8M3 { double m[3][3]; };
Vec3d operator*(const P8M3&, const Vec3d&);
Vec3d p8_apply(const P8M3&, const Vec3d&);
P8M3 p8_to_octree_rotation();
void p8_rotate_to_world(Octree*);
}} // namespace

#ifdef P8_SINGLE_TU
namespace Slic3r { namespace FillAdaptive {
template<typename Vector>
bool triangle_AABB_intersects(const Vector &a, const Vector &b, const Vector &c, const BoundingBoxf3 &aabb);
#include "p8body.inc"
}} // namespace body close
#endif
#endif
