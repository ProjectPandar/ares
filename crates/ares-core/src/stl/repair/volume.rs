use super::{Facet, normals};

// admesh/util.cpp::get_volume: source-order float accumulation around vertex 0.
pub(super) fn calculate(facets: &[Facet]) -> f32 {
    let reference = facets[0].vertices[0];
    let mut volume = 0.0_f32;
    for facet in facets {
        let point = facet.vertices[0];
        let height = normals::dot(
            facet.normal,
            [
                point.x - reference.x,
                point.y - reference.y,
                point.z - reference.z,
            ],
        );
        volume += (area(facet) * height) / 3.0;
    }
    volume
}

// get_area casts coordinates before products, but converts each sum back to float.
fn area(facet: &Facet) -> f32 {
    let vertices = facet
        .vertices
        .map(|point| [f64::from(point.x), f64::from(point.y), f64::from(point.z)]);
    let cross: [[f64; 3]; 3] = std::array::from_fn(|index| {
        let a = vertices[index];
        let b = vertices[(index + 1) % 3];
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
    });
    let sum = std::array::from_fn(|axis| (cross[0][axis] + cross[1][axis] + cross[2][axis]) as f32);
    0.5 * normals::dot(normals::calculate(facet), sum)
}
