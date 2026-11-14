//! `orientation::AutoOrienter` (`Orient.cpp:78-572`). The input mesh is borrowed;
//! selecting a rotation is separate from applying it to a project or instance.
mod candidates;
mod cooling;
mod mesh;
mod project;
pub(crate) use project::apply_cli_orientation;
pub use project::{auto_orient_instance, auto_orient_object};
mod score;
mod types;
use super::eigen::{euler_angles_210, rotation_from_two_vectors};
use crate::geometry::mesh_normals::dot;
use crate::{ProjectMesh, SliceError};
pub use types::{CoolingOrientation, OrientMesh, OrientParams, Orientation};

/// Raw `TriangleMesh*` constructor: source defaults, including ASCENT=-0.5.
pub fn auto_orient_mesh(mesh: &ProjectMesh) -> Result<Orientation, SliceError> {
    process(mesh, &OrientParams::default(), None)
}

/// `_orient` class inputs in request order. Scheduling independent requests may
/// run externally; each request always calculates the same optional outputs.
pub fn auto_orient(
    meshes: &[OrientMesh<'_>],
    params: &OrientParams,
) -> Result<Vec<Orientation>, SliceError> {
    meshes
        .iter()
        .map(|item| {
            let mut params = *params;
            params.ascent = (std::f64::consts::PI
                - item.overhang_angle * std::f64::consts::PI / 180.)
                .cos() as f32;
            process(item.mesh, &params, item.cooling_direction)
        })
        .collect()
}

fn process(
    input: &ProjectMesh,
    params: &OrientParams,
    machine: Option<[f64; 3]>,
) -> Result<Orientation, SliceError> {
    let mesh = mesh::Mesh::new(input)?;
    let mut results: Vec<_> = candidates::directions(&mesh)
        .into_iter()
        .map(|direction| score::evaluate(&mesh, direction, params, machine.is_some()))
        .collect();
    results.sort_by(|a, b| a.score.total_cmp(&b.score));
    let mut best = 0;
    for i in 1..results.len() - 1 {
        if f64::from((results[i].score - results[0].score).abs()) < 1e-4
            && f64::from((dot(results[0].direction, [0., 0., 1.]) - 1.).abs()) > 1e-4
        {
            if f64::from((dot(results[i].direction, [0., 0., 1.]) - 1.).abs()) < 1e-8 {
                best = i;
                break;
            }
        } else {
            break;
        }
    }
    let selected = results.swap_remove(best);
    let direction = selected.direction.map(f64::from);
    let (axis, angle, rotation_matrix) = rotation_from_two_vectors(direction, [0., 0., 1.]);
    let angles = euler_angles_210(rotation_matrix);
    let euler_angles = [angles[2], angles[1], angles[0]];
    let cooling = machine.map(|machine| {
        let direction = cooling::direction(&mesh, euler_angles, &selected.cooling, machine);
        let (axis, angle, rotation_matrix) = rotation_from_two_vectors(direction, machine);
        CoolingOrientation {
            direction,
            axis,
            angle,
            rotation_matrix,
        }
    });
    Ok(Orientation {
        direction,
        axis,
        angle,
        rotation_matrix,
        euler_angles,
        cooling,
    })
}
