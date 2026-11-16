//! `TriangleMesh.hpp:349-354` and `admesh/stl.h:240-243`.
pub(crate) fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    // Eigen's fixed-size three-element scalar reduction splits 1 + 2.
    a[0] * b[0] + (a[1] * b[1] + a[2] * b[2])
}
pub(crate) fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub(crate) fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn normalized(n: [f32; 3]) -> [f32; 3] {
    let squared = dot(n, n);
    if squared > 0. {
        n.map(|value| value / squared.sqrt())
    } else {
        n
    }
}
pub(crate) fn face_normal(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> [f32; 3] {
    normalized(normalized(cross(sub(b, a), sub(c, b))))
}
pub(crate) fn facet_area(a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let n = cross(sub(a, b), sub(a, c));
    dot(n, n).sqrt().abs() / 2.
}
