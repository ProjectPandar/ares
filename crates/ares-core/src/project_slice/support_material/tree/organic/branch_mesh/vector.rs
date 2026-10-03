//! Scalar boundaries of the native Eigen three-component operations.

pub(super) fn distance_squared(first: [f32; 3], second: [f32; 3]) -> f32 {
    let delta = std::array::from_fn::<_, 3, _>(|axis| first[axis] - second[axis]);
    delta[0] * delta[0] + (delta[1] * delta[1] + delta[2] * delta[2])
}

pub(super) fn normalized_f32(vector: [f32; 3]) -> [f32; 3] {
    let squared = vector[0] * vector[0] + (vector[1] * vector[1] + vector[2] * vector[2]);
    if squared > 0.0 {
        let length = squared.sqrt();
        vector.map(|value| value / length)
    } else {
        vector
    }
}

pub(super) fn normalized_f64(vector: [f64; 3]) -> [f64; 3] {
    let squared = (vector[0] * vector[0] + vector[1] * vector[1]) + vector[2] * vector[2];
    if squared > 0.0 {
        let length = squared.sqrt();
        vector.map(|value| value / length)
    } else {
        vector
    }
}

pub(super) fn cross(first: [f32; 3], second: [f32; 3]) -> [f32; 3] {
    [
        first[1] * second[2] - first[2] * second[1],
        first[2] * second[0] - first[0] * second[2],
        first[0] * second[1] - first[1] * second[0],
    ]
}
