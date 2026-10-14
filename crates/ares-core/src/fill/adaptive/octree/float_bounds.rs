// FillAdaptive.cpp:1495-1497: BoundingBox3Base<Vec3f>. BoundingBox.cpp:
// 182-185 size and 232-235 center complete in float before widening.
pub(super) fn of(vertices: &[[f64; 3]]) -> ([f64; 3], f64) {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for vertex in vertices {
        for axis in 0..3 {
            let value = vertex[axis] as f32;
            min[axis] = min[axis].min(value);
            max[axis] = max[axis].max(value);
        }
    }
    let center = std::array::from_fn(|axis| f64::from((min[axis] + max[axis]) / 2.0));
    let size = (max[0] - min[0]).max(max[1] - min[1]).max(max[2] - min[2]);
    (center, f64::from(size))
}
