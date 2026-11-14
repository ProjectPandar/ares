//! `Orient.cpp:346-373,402-510`: float projection, feature extraction and scoring.
use super::{OrientParams, mesh::Mesh};
use crate::geometry::mesh_normals::dot;

pub(super) struct Scored {
    pub score: f32,
    pub direction: [f32; 3],
    pub cooling: Vec<f32>,
}
pub(super) fn evaluate(
    mesh: &Mesh,
    direction: [f32; 3],
    p: &OrientParams,
    cooling: bool,
) -> Scored {
    let projected: Vec<_> = mesh
        .facets
        .iter()
        .map(|f| f.vertices.map(|v| dot(v, direction)))
        .collect();
    let min_z = projected
        .iter()
        .flatten()
        .copied()
        .fold(f32::INFINITY, f32::min);
    // Eigen's Array comparison takes `const Scalar&`: the double EPSILON
    // expression is converted back to float before comparing coefficients.
    let bottom_limit = (f64::from(min_z + p.first_layer_height) - 1e-4) as f32;
    let half_limit = (f64::from(min_z + p.first_layer_height / 2.) - 1e-4) as f32;
    let mut bottom = Vec::with_capacity(projected.len());
    let mut bottom_half = Vec::with_capacity(projected.len());
    let mut overhang = Vec::with_capacity(projected.len());
    let mut low_angle = Vec::with_capacity(projected.len());
    let mut cooling_areas = if cooling {
        Vec::with_capacity(projected.len())
    } else {
        Vec::new()
    };
    for (facet, z) in mesh.facets.iter().zip(&projected) {
        let max = z[0].max(z[1]).max(z[2]);
        let on_bottom = max < bottom_limit;
        let on_half = max < half_limit;
        bottom.push(if on_bottom { facet.area } else { 0. });
        bottom_half.push(if on_half { facet.area } else { 0. });
        let projection = dot(facet.normal, direction);
        let appearance_area =
            facet.area * (f32::from(u8::from(facet.appearance)) * p.appearance_face_support + 1.);
        let overhang_area = if projection < p.ascent && !on_half {
            appearance_area
        } else {
            0.
        };
        overhang.push(if p.min_volume {
            let height = (z[0] + z[1] + z[2]) / 3. - min_z;
            height * overhang_area * (projection - p.ascent).min(0.).abs()
        } else {
            overhang_area.abs()
        });
        low_angle.push(
            if projection.abs() < p.low_angle_max
                && projection.abs() > p.low_angle_min
                && max > min_z + p.first_layer_height
            {
                facet.area
            } else {
                0.
            },
        );
        if cooling {
            let area = if projection < -0.6427 && projection > -0.97 && !on_half {
                facet.area
            } else {
                0.
            };
            cooling_areas.push(area * (1. - projection));
        }
    }
    let bottom_area =
        (f64::from(selected_sum(&bottom)) * 0.5 + f64::from(selected_sum(&bottom_half))) as f32;
    let hull_bottom: Vec<_> = mesh
        .hull
        .iter()
        .map(|f| {
            let z = f.vertices.map(|v| dot(v, direction));
            if z[0].max(z[1]).max(z[2]) < bottom_limit {
                f.area
            } else {
                0.
            }
        })
        .collect();
    let bottom_hull = selected_sum(&hull_bottom);
    let contour = 4. * bottom_area.sqrt();
    let overhang = vector_sum(&overhang);
    let laf = vector_sum(&low_angle);
    let denominator =
        p.tar_d + p.contour_f * contour + p.bottom_f * bottom_area + p.bottom_hull_f * bottom_hull;
    let numerator = overhang * p.tar_c
        + p.tar_d
        + p.tar_low_angle * laf * f32::from(u8::from(p.use_low_angle_face));
    let mut score = if p.min_volume {
        let overhang = overhang / 25.;
        p.tar_a * (overhang + p.tar_b)
            + p.relative_f
                * (overhang * p.tar_c
                    + p.tar_d
                    + p.tar_low_angle * laf * f32::from(u8::from(p.use_low_angle_face)))
                / (denominator + p.tar_e * overhang + p.tar_projected_area * 0.)
    } else {
        p.relative_f * numerator / (denominator + p.tar_projected_area * 0.)
    };
    score += f32::from(u8::from(bottom_area < p.bottom_min)) * 100.;
    Scored {
        score,
        direction,
        cooling: cooling_areas,
    }
}

fn selected_sum(values: &[f32]) -> f32 {
    values.iter().copied().reduce(|a, b| a + b).unwrap_or(0.)
}

/// Eigen dynamic VectorXf SSE2 reduction, expressed portably with fixed lanes.
fn vector_sum(values: &[f32]) -> f32 {
    if values.len() < 4 {
        return selected_sum(values);
    }
    let packets = values.len() / 4;
    let mut first: [f32; 4] = values[..4].try_into().expect("one packet");
    if packets > 1 {
        let mut second: [f32; 4] = values[4..8].try_into().expect("second packet");
        for pair in values[8..packets / 2 * 8].as_chunks::<8>().0 {
            for lane in 0..4 {
                first[lane] += pair[lane];
                second[lane] += pair[lane + 4];
            }
        }
        for lane in 0..4 {
            first[lane] += second[lane];
        }
        if !packets.is_multiple_of(2) {
            for lane in 0..4 {
                first[lane] += values[(packets - 1) * 4 + lane];
            }
        }
    }
    let mut result = (first[0] + first[2]) + (first[1] + first[3]);
    for &value in &values[packets * 4..] {
        result += value;
    }
    result
}
