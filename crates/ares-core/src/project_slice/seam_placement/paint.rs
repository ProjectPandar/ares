//! SeamPlacer.cpp::gather_enforcers_blockers.

mod contour;
mod intervals;
mod samples;

use crate::{
    Point3d, ProcessSeamPosition, Project, ProjectObject, ProjectVolumeType, SliceError,
    geometry::CoordinateScale,
    project::{
        effective_config::types::{BoundedResolvedProjectConfig, ResolvedPrintObjectConfig},
        facets_annotation::strict_annotated_facets,
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
            || volumes[0]
                .mesh()
                .seam_facets
                .codes
                .iter()
                .any(|code| !matches!(code, 0 | 4 | 8))
        {
            return Err(deferred());
        }
    }
    Ok(())
}

fn deferred() -> SliceError {
    SliceError::InvalidInput(
        "seam painting is outside the supported aligned single-part leaf domain".to_owned(),
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
        if mesh.seam_facets.codes.len() == mesh.triangles().len()
            && mesh.seam_facets.codes.iter().all(|&code| code == 4)
        {
            return Err(deferred());
        }
        let transform = centered_transform(object, group, scale).then(volume.transform());
        let surface = |state| -> Result<PaintSurface, SliceError> {
            // Ordinary and strict leaves coincide without selector subdivisions.
            let selected = strict_annotated_facets(mesh, &mesh.seam_facets, state)?;
            let vertices = selected
                .vertices
                .into_iter()
                .map(|[x, y, z]| {
                    let point =
                        transform.transform_point(Point3d::new(x.into(), y.into(), z.into()));
                    Vec3::new(point.x as f32, point.y as f32, point.z as f32)
                })
                .collect::<Vec<_>>();
            let triangles = selected
                .triangles
                .iter()
                .map(|face| Triangle {
                    vertices: face.map(|vertex| vertices[vertex as usize]),
                })
                .collect();
            let mesh = TriangleMesh {
                triangles,
                negative_start: 0,
            };
            let tree = TriangleBvh::new(&mesh);
            Ok(PaintSurface {
                mesh,
                tree,
                contour: contour::edges(&vertices, &selected.triangles),
            })
        };
        Ok(Some(Self {
            enforcers: surface(1)?,
            blockers: surface(2)?,
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
