//! `Print.cpp:2995–3017::first_layer_islands` and `Brim.cpp:918–924`.
use super::super::{layers::Schedule, tool_ordering};
use super::coverage::covered_by_spacing;
use crate::{
    ExtrusionRole, ProcessSupportType, SliceError,
    geometry::{ClipperError, CoordinateScale, ExPolygon, Point},
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::traversal::PreparedPostClassicTraversal,
        raft::RaftLayerPlan,
        support_material::{InstalledSupportLayer, SupportExtrusionEntity, SupportExtrusionPath},
    },
};

pub(in crate::project_slice::gcode_emit) struct FirstLayerIslands {
    pub(in crate::project_slice::gcode_emit) plate: Vec<Point>,
    pub(in crate::project_slice::gcode_emit) support_spacing: Vec<Vec<Point>>,
}

impl FirstLayerIslands {
    pub(in crate::project_slice::gcode_emit) fn collect(
        traversal: &PreparedPostClassicTraversal,
        support: &[Vec<InstalledSupportLayer>],
        schedule: &Schedule,
    ) -> Result<Self, SliceError> {
        let mut plate = Vec::new();
        let mut support_spacing = Vec::new();
        let scale = traversal.scale;
        for (index, object) in traversal.objects.iter().enumerate() {
            let mut local = object
                .slices(0)
                .into_iter()
                .flatten()
                .flat_map(|polygon| polygon.contour().points().iter().copied())
                .collect::<Vec<_>>();
            let first = first_support(index, support, schedule);
            let spacing = first
                .map(|first| first.coverage(0.0, scale))
                .transpose()
                .map_err(geometry_error)?
                .unwrap_or_default();
            if let Some(first) = first {
                local.extend(
                    first
                        .islands(
                            tool_ordering::object_options(traversal, index).support_type,
                            scale,
                        )
                        .map_err(geometry_error)?,
                );
            }
            for copy in &schedule.instances[index] {
                plate.extend(
                    local
                        .iter()
                        .map(|p| Point::new(p.x() + copy.shift.x(), p.y() + copy.shift.y())),
                );
            }
            support_spacing.push(spacing);
        }
        Ok(Self {
            plate,
            support_spacing,
        })
    }
}

#[derive(Clone, Copy)]
enum Support<'a> {
    Installed(&'a InstalledSupportLayer),
    Raft(&'a RaftLayerPlan, &'a OrderedExtrusionLayer),
}
fn first_support<'a>(
    index: usize,
    support: &'a [Vec<InstalledSupportLayer>],
    schedule: &'a Schedule,
) -> Option<Support<'a>> {
    let installed = support[index].first();
    let raft = schedule.raft.as_ref().and_then(|raft| {
        raft.plans[index]
            .first()
            .map(|plan| (plan, &raft.layers[index][0]))
    });
    if let Some((plan, layer)) =
        raft.filter(|(plan, _)| installed.is_none_or(|layer| plan.z.print_z < layer.print_z))
    {
        Some(Support::Raft(plan, layer))
    } else {
        installed.map(Support::Installed)
    }
}
impl Support<'_> {
    fn islands(
        self,
        kind: ProcessSupportType,
        scale: CoordinateScale,
    ) -> Result<Vec<Point>, ClipperError> {
        match kind {
            ProcessSupportType::TreeAuto | ProcessSupportType::TreeManual => Ok(self.contours()),
            ProcessSupportType::NormalAuto | ProcessSupportType::NormalManual => {
                self.coverage((1e-4 / scale.factor()) as f32, scale)
            }
        }
    }
    fn contours(self) -> Vec<Point> {
        match self {
            Self::Installed(layer) => layer
                .slices
                .iter()
                .flat_map(|polygon| polygon.contour().points().iter().copied())
                .collect(),
            Self::Raft(plan, _) => plan
                .polygons
                .iter()
                .flat_map(|polygon| polygon.points().iter().copied())
                .collect(),
        }
    }
    fn coverage(self, epsilon: f32, scale: CoordinateScale) -> Result<Vec<Point>, ClipperError> {
        let mut output = Vec::new();
        match self {
            Self::Installed(layer) => {
                for entity in &layer.support_fills {
                    support_entity(&mut output, entity, epsilon, scale)?;
                }
            }
            Self::Raft(_, layer) => {
                for entity in layer.islands.iter().flat_map(|island| &island.entities) {
                    raft_entity(&mut output, entity, epsilon, scale)?;
                }
            }
        }
        Ok(output)
    }
}
fn support_entity(
    output: &mut Vec<Point>,
    entity: &SupportExtrusionEntity,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    match entity {
        SupportExtrusionEntity::Path(path) | SupportExtrusionEntity::Loop(path) => {
            support_path(output, path, epsilon, scale)?
        }
        SupportExtrusionEntity::MultiPath(paths) => {
            for path in paths {
                support_path(output, path, epsilon, scale)?;
            }
        }
        SupportExtrusionEntity::NoSort(entities) => {
            for entity in entities {
                support_entity(output, entity, epsilon, scale)?;
            }
        }
    }
    Ok(())
}
fn support_path(
    output: &mut Vec<Point>,
    path: &SupportExtrusionPath,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let points = path
        .polyline
        .points
        .iter()
        .map(|point| Point::new(point.x, point.y))
        .collect::<Vec<_>>();
    append(
        output,
        points,
        spacing(path.width, path.height, path.role),
        epsilon,
        scale,
    )
}
fn raft_entity(
    output: &mut Vec<Point>,
    entity: &IslandPrintEntity,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    match entity {
        IslandPrintEntity::Fill(entity) => raft_path(output, entity, epsilon, scale)?,
        IslandPrintEntity::FillCollection(collection) => {
            for entity in &collection.entities {
                raft_path(output, entity, epsilon, scale)?;
            }
        }
        IslandPrintEntity::Perimeter(_) | IslandPrintEntity::Thin(_) => {
            unreachable!("raft materialization emits support fills")
        }
    }
    Ok(())
}
fn raft_path(
    output: &mut Vec<Point>,
    entity: &FillExtrusionEntity,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => append(
            output,
            path.polyline
                .points()
                .iter()
                .map(|point| Point::new(point.x, point.y)),
            spacing(path.width, path.height, path.role),
            epsilon,
            scale,
        ),
        FillExtrusionEntity::VariableWidth(_) => {
            unreachable!("raft materialization emits constant-width support fills")
        }
    }
}
fn spacing(width: f32, height: f32, role: ExtrusionRole) -> f32 {
    if role.is_bridge() {
        (f64::from(width) + 0.05) as f32
    } else {
        width - height * (1.0 - std::f64::consts::FRAC_PI_4) as f32
    }
}
fn append(
    output: &mut Vec<Point>,
    points: impl IntoIterator<Item = Point>,
    spacing: f32,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    for polygon in covered_by_spacing(points, spacing, epsilon, scale)? {
        output.extend_from_slice(polygon.points());
    }
    Ok(())
}
fn geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("first-layer support coverage: {error:?}"))
}

/// `Brim.cpp:590-634`: support contours that the brim must avoid. Tree
/// support contributes its first-layer slice contours (`lslices`),
/// inner-normal support contributes the spacing coverage of its fills
/// (`polygons_covered_by_spacing`).
pub(in crate::project_slice::gcode_emit) fn support_no_brim(
    inner_tree: bool,
    layer: &InstalledSupportLayer,
    scale: CoordinateScale,
) -> Result<Vec<ExPolygon>, ClipperError> {
    match inner_tree {
        true => Ok(layer
            .support_islands
            .iter()
            .map(|polygon| ExPolygon::new(polygon.contour().clone(), Vec::new()))
            .collect()),
        false => {
            // `ExtrusionEntity.cpp:75-84` buffers by spacing/2 + the
            // caller-scaled SCALED_EPSILON (= scaled 1e-4, 100 units at
            // factor 1e6), not a millimetre value.
            let epsilon = (1e-4 * scale.factor()) as f32;
            let mut polygons = Vec::new();
            for entity in &layer.support_fills {
                no_brim_entity(&mut polygons, entity, epsilon, scale)?;
            }
            Ok(polygons
                .into_iter()
                .map(|polygon| ExPolygon::new(polygon, Vec::new()))
                .collect())
        }
    }
}
fn no_brim_entity(
    output: &mut Vec<crate::geometry::Polygon>,
    entity: &SupportExtrusionEntity,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    match entity {
        SupportExtrusionEntity::Path(path) | SupportExtrusionEntity::Loop(path) => {
            no_brim_path(output, path, epsilon, scale)
        }
        SupportExtrusionEntity::MultiPath(paths) => {
            for path in paths {
                no_brim_path(output, path, epsilon, scale)?;
            }
            Ok(())
        }
        SupportExtrusionEntity::NoSort(entities) => {
            for entity in entities {
                no_brim_entity(output, entity, epsilon, scale)?;
            }
            Ok(())
        }
    }
}
fn no_brim_path(
    output: &mut Vec<crate::geometry::Polygon>,
    path: &SupportExtrusionPath,
    epsilon: f32,
    scale: CoordinateScale,
) -> Result<(), ClipperError> {
    let points = path
        .polyline
        .points
        .iter()
        .map(|point| Point::new(point.x, point.y));
    output.extend(covered_by_spacing(
        points,
        spacing(path.width, path.height, path.role),
        epsilon,
        scale,
    )?);
    Ok(())
}
