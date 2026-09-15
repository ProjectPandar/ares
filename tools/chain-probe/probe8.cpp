#define P8_SINGLE_TU
// probe8: measure the upstream FillAdaptive line pitch for a 10mm cube at
// candidate spacings — identify which spacing orca actually uses.
#include "p8shim.hpp"
#include <cstdio>
#include "p8sat.inc"
#include "p8helpers.inc"

using namespace Slic3r::FillAdaptive;
using Slic3r::FillAdaptive::P8Mesh;

// Simple 3x3 rotation product (AngleAxis equivalents of octree_rot).
struct M3 { double m[3][3]; };
static M3 axis_rot(int axis, double ang) {
    double s = std::sin(ang), c = std::cos(ang);
    if (axis == 0) return {{{1,0,0},{0,c,-s},{0,s,c}}};
    if (axis == 1) return {{{c,0,s},{0,1,0},{-s,0,c}}};
    return {{{c,-s,0},{s,c,0},{0,0,1}}};
}
static M3 mul(const M3&a, const M3&b) {
    M3 o{};
    for (int r=0;r<3;r++) for (int c=0;c<3;c++)
        o.m[r][c] = a.m[r][0]*b.m[0][c] + a.m[r][1]*b.m[1][c] + a.m[r][2]*b.m[2][c];
    return o;
}
static Vec3d apply(const M3&a, const Vec3d&v) {
    return Vec3d(a.m[0][0]*v.x()+a.m[0][1]*v.y()+a.m[0][2]*v.z(),
                 a.m[1][0]*v.x()+a.m[1][1]*v.y()+a.m[1][2]*v.z(),
                 a.m[2][0]*v.x()+a.m[2][1]*v.y()+a.m[2][2]*v.z());
}

int main(int argc, char **argv) {
    double spacing = argc > 1 ? std::atof(argv[1]) : 25.2;
    double z = argc > 2 ? std::atof(argv[2]) : 1.8;

    // 10mm cube centered at the origin (trafo_centered equivalent).
    const double h = 5.0;
    // Upstream trafo_centered centers XY only — the mesh keeps its world z
    // (here z in [0,10] for the cube10 fixture).
    const double hz = 5.0;
    Vec3d verts[8] = {
        {-h,-h,-5.0},{h,-h,-5.0},{h,h,-5.0},{-h,h,-5.0},
        {-h,-h,hz},{h,-h,hz},{h,h,hz},{-h,h,hz}};
    int tris[12][3] = {
        {0,1,2},{0,2,3},{4,6,5},{4,7,6},
        {0,4,5},{0,5,1},{1,5,6},{1,6,2},
        {2,6,7},{2,7,3},{3,7,4},{3,4,0}};

    // to_octree rotation: Rx(-r0)*Ry(-r1)*Rz(-r2)
    const double r0 = 5.0*M_PI/4.0, r1 = 215.264*M_PI/180.0, r2 = M_PI/6.0;
    M3 to_octree = mul(mul(axis_rot(0,-r0), axis_rot(1,-r1)), axis_rot(2,-r2));
    M3 to_world = mul(mul(axis_rot(2,r2), axis_rot(1,r1)), axis_rot(0,r0));

    Vec3d rot[8];
    for (int i=0;i<8;i++) rot[i] = apply(to_octree, verts[i]);

    P8Mesh mesh;
    mesh.vertices.assign(rot, rot + 8);
    for (int i = 0; i < 12; ++i) mesh.indices.push_back({tris[i][0], tris[i][1], tris[i][2]});
    OctreePtr octree = build_octree(mesh, {}, spacing, false);

    for (int dir = 0; dir < 3; dir++) {
        FillContext ctx(*octree, z, dir);
        generate_infill_lines_recursive(ctx, octree->root_cube, 0, int(octree->cubes_properties.size())-1);
        for (const Line &l : ctx.output_lines)
            std::printf("D%d %f %f -> %f %f\n", dir,
                l.a.x()/40.0, l.a.y()/40.0, l.b.x()/40.0, l.b.y()/40.0);
        for (const Line &l : ctx.temp_lines)
            if (l.a.x() != std::numeric_limits<coord_t>::max())
                std::printf("D%d %f %f -> %f %f\n", dir,
                    l.a.x()/40.0, l.a.y()/40.0, l.b.x()/40.0, l.b.y()/40.0);
    }
    return 0;
}
