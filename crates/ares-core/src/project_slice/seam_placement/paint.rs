//! SeamPlacer.cpp::gather_enforcers_blockers.

mod contour;
mod intervals;
mod samples;

use crate::{
    Point3d, ProcessSeamPosition, Project, ProjectObject, ProjectVolumeType, SliceError,
    geometry::CoordinateScale,
    project::{
        effective_config::types::{BoundedResolvedProjectConfig, ResolvedPrintObjectConfig},
        facets_annotation::FacetSelector,
    },
};

use super::{
    mesh::{Triangle, TriangleMesh, Vec3, centered_transform},
    spatial::TriangleBvh,
};

pub(super) use samples::OVERSAMPLING;
pub(in crate::project_slice) use samples::paint_candidates;

pub(in crate::project_slice) fn validate_paint(
    project: &Project,
    resolved: &BoundedResolvedProjectConfig,
) -> Result<(), SliceError> {
    for object in &resolved.objects {
        let volumes = project.objects()[object.source_object_index].volumes();
        if volumes
            .iter()
            .all(|volume| volume.mesh().seam_facets.mapping.is_empty())
        {
            continue;
        }
        if volumes.len() != 1
            || volumes[0].volume_type() != ProjectVolumeType::ModelPart
            || object.object.seam_position != ProcessSeamPosition::Aligned
            || resolved.views.full.process.print.spiral_mode.0
            || volumes[0].mesh().seam_facets.used_states()[3..]
                .iter()
                .any(|&used| used)
        {
            return Err(deferred());
        }
    }
    Ok(())
}

fn deferred() -> SliceError {
    SliceError::InvalidInput(
        "seam painting is outside the supported aligned single-part domain".to_owned(),
    )
}

pub(in crate::project_slice) struct SeamPaint {
    enforcers: PaintSurface,
    blockers: PaintSurface,
}

impl SeamPaint {
    pub(super) fn from_object(
        object: &ProjectObject,
        group: &ResolvedPrintObjectConfig,
        scale: CoordinateScale,
    ) -> Result<Option<Self>, SliceError> {
        let Some(volume) = object
            .volumes()
            .iter()
            .find(|volume| !volume.mesh().seam_facets.mapping.is_empty())
        else {
            return Ok(None);
        };
        let mesh = volume.mesh();
        let annotation = &mesh.seam_facets;
        let states = annotation.used_states();
        if annotation.mapping.len() == mesh.triangles().len() && !states[0] && !states[2] {
            return Err(deferred());
        }
        let selector = FacetSelector::deserialize(mesh, annotation)?;
        let transform = centered_transform(object, group, scale).then(volume.transform());
        let transform_vertices = |vertices: Vec<[f32; 3]>| -> Vec<Vec3> {
            vertices
                .into_iter()
                .map(|[x, y, z]| {
                    let point =
                        transform.transform_point(Point3d::new(x.into(), y.into(), z.into()));
                    Vec3::new(point.x as f32, point.y as f32, point.z as f32)
                })
                .collect()
        };
        let surface = |state| -> PaintSurface {
            let selected = selector.ordinary_facets(state);
            let vertices = transform_vertices(selected.vertices);
            let triangles = selected
                .triangles
                .into_iter()
                .map(|face| Triangle {
                    vertices: face.map(|vertex| vertices[vertex as usize]),
                })
                .collect();
            let mesh = TriangleMesh {
                triangles,
                negative_start: 0,
            };
            let tree = TriangleBvh::new(&mesh);
            let strict = selector.strict_facets(state);
            let contour = contour::edges(&transform_vertices(strict.vertices), &strict.triangles);
            PaintSurface {
                mesh,
                tree,
                contour,
            }
        };
        Ok(Some(Self {
            enforcers: surface(1),
            blockers: surface(2),
        }))
    }
}

struct PaintSurface {
    mesh: TriangleMesh,
    tree: TriangleBvh,
    contour: Vec<contour::PaintEdge>,
}

impl PaintSurface {
    fn near(&self, position: Vec3, radius: f32) -> bool {
        self.tree.any_in_radius(&self.mesh, position, radius)
    }
}
