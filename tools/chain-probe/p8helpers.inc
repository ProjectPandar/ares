namespace Slic3r { namespace FillAdaptive {

P8M3 p8_axis_rot(int axis, double ang) {
    double s = std::sin(ang), c = std::cos(ang);
    if (axis == 0) return {{{1,0,0},{0,c,-s},{0,s,c}}};
    if (axis == 1) return {{{c,0,s},{0,1,0},{-s,0,c}}};
    return {{{c,-s,0},{s,c,0},{0,0,1}}};
}
P8M3 p8_mul(const P8M3&a, const P8M3&b) {
    P8M3 o{};
    for (int r=0;r<3;r++) for (int c=0;c<3;c++)
        o.m[r][c] = a.m[r][0]*b.m[0][c] + a.m[r][1]*b.m[1][c] + a.m[r][2]*b.m[2][c];
    return o;
}
Vec3d p8_apply(const P8M3&a, const Vec3d&v) {
    return Vec3d(a.m[0][0]*v.x()+a.m[0][1]*v.y()+a.m[0][2]*v.z(),
                 a.m[1][0]*v.x()+a.m[1][1]*v.y()+a.m[1][2]*v.z(),
                 a.m[2][0]*v.x()+a.m[2][1]*v.y()+a.m[2][2]*v.z());
}
Vec3d operator*(const P8M3 &a, const Vec3d &v) { return p8_apply(a, v); }
P8M3 p8_to_octree_rotation() {
    const double r0 = 5.0*M_PI/4.0, r1 = 215.264*M_PI/180.0, r2 = M_PI/6.0;
    return p8_mul(p8_mul(p8_axis_rot(0,-r0), p8_axis_rot(1,-r1)), p8_axis_rot(2,-r2));
}
P8M3 p8_to_world_rotation() {
    const double r0 = 5.0*M_PI/4.0, r1 = 215.264*M_PI/180.0, r2 = M_PI/6.0;
    return p8_mul(p8_mul(p8_axis_rot(2,r2), p8_axis_rot(1,r1)), p8_axis_rot(0,r0));
}
void p8_rotate_to_world(Octree *octree) {
    P8M3 rot = p8_to_world_rotation();
    transform_center(octree->root_cube, rot);
    octree->origin = rot * octree->origin;
}

}} // namespace Slic3r::FillAdaptive
