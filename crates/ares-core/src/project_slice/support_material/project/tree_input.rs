//! PrintObject/Print views required by `TreeSupport.cpp::generate`.

use super::region::{self, Region};
use crate::{
    FloatOrPercent, Point3d, ProcessBrimType, ProcessDraftShield, ProjectObject, ProjectSettings,
    ProjectVolumeType, SliceError,
    fill::lightning::generator::SupportSettings,
    geometry::{BoundingBox, Point, Polygon},
    project::effective_config::types::ResolvedProjectObject,
    project_slice::{
        layers::PlannedPrintObject,
        perimeters::flow::build_nonbridging_flow,
        support_material::{
            install_layers::InstalledSupportLayer,
            tree::{NonOrganicInput, TreeInput, VerticalEnforcer, generate_non_organic},
            trim_object::TrimRegion,
        },
    },
};

pub(super) struct ProjectInputs<'a> {
    pub(super) source: &'a ProjectObject,
    pub(super) resolved: &'a ResolvedProjectObject,
    pub(super) printing_objects: &'a [ResolvedProjectObject],
    pub(super) plan: &'a PlannedPrintObject,
    pub(super) data: &'a [Vec<Region<'a>>],
    pub(super) tops: &'a [Vec<Polygon>],
    pub(super) trim_regions: &'a [Vec<TrimRegion<'a>>],
    pub(super) settings: &'a ProjectSettings,
    pub(super) max_object_layers: usize,
    pub(super) vertical_enforcers: &'a [VerticalEnforcer],
}

pub(super) fn generate(
    tree: &TreeInput<'_>,
    project: &ProjectInputs<'_>,
) -> Result<Vec<InstalledSupportLayer>, SliceError> {
    let bottoms = project
        .data
        .iter()
        .map(|regions| {
            regions
                .iter()
                .flat_map(|region| &region.bottoms)
                .flat_map(|polygon| std::iter::once(polygon.contour()).chain(polygon.holes()))
                .cloned()
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let print = &project.settings.process.print;
    let first = &project.printing_objects[0];
    let first_region = &first.all_regions[0];
    let width = if !print.initial_layer_line_width.is_non_positive() {
        print.initial_layer_line_width
    } else if !first_region.inner_wall_line_width.is_non_positive() {
        first_region.inner_wall_line_width
    } else {
        first.object.line_width
    };
    let brim_flow = build_nonbridging_flow(
        width,
        print.initial_layer_print_height.0 as f32,
        region::nozzle(first_region.outer_wall_filament_id.0, tree.nozzles) as f32,
    )?;
    let has_brim = project.printing_objects.iter().any(|resolved| {
        let object = &resolved.object;
        object.raft_layers.0 <= 0
            && (object.brim_type == ProcessBrimType::AutoBrim
                || (object.brim_type != ProcessBrimType::NoBrim && object.brim_width.0 > 0.0))
    });
    let infinite = print.draft_shield == ProcessDraftShield::Enabled && print.skirt_loops.0 > 0;
    let skirt_layers = if infinite {
        project.max_object_layers
    } else {
        (print.skirt_height.0 as usize).min(project.max_object_layers)
    };
    let max_nozzle = tree
        .nozzles
        .0
        .iter()
        .map(|nozzle| nozzle.0)
        .fold(f64::NEG_INFINITY, f64::max);
    let region = &project.resolved.all_regions[0];
    generate_non_organic(&NonOrganicInput {
        tree,
        planned: &project.plan.layers,
        tops: project.tops,
        bottoms: &bottoms,
        trim_regions: project.trim_regions,
        bounds: mesh_bounds(project, tree.scale.factor()),
        vertical_enforcers: project.vertical_enforcers,
        independent: print.independent_support_layer_height.0,
        brim_flow,
        brim_skirt_layers: usize::from(has_brim).max(skirt_layers),
        lightning_settings: SupportSettings {
            layer_height: tree.object.layer_height.0,
            infill_line_width: absolute(region.sparse_infill_line_width, max_nozzle),
            line_width: absolute(tree.object.line_width, max_nozzle),
            max_nozzle_diameter: max_nozzle,
            scale: tree.scale,
        },
    })
}

// PrintObject.cpp89-113 and Print.hpp347: centered integer size, not slice extents.
fn mesh_bounds(input: &ProjectInputs<'_>, factor: f64) -> BoundingBox {
    let first = input.source.instances()[0]
        .transform()
        .without_translation();
    let mut points = input
        .source
        .volumes()
        .iter()
        .filter(|volume| {
            volume.volume_type() == ProjectVolumeType::ModelPart
                && !volume.mesh().triangles().is_empty()
        })
        .flat_map(|volume| {
            let transform = first.then(volume.transform());
            volume
                .mesh()
                .vertices()
                .iter()
                .map(move |&point| transform.transform_point(point))
        });
    let point = points.next().expect("printing object has model vertices");
    let (mut min, mut max) = ([point.x, point.y], [point.x, point.y]);
    for point in points {
        min[0] = min[0].min(point.x);
        min[1] = min[1].min(point.y);
        max[0] = max[0].max(point.x);
        max[1] = max[1].max(point.y);
    }
    let (x, y) = first.first_xy_column();
    let (target_x, target_y) = input.resolved.print_objects[input.plan.transform_index]
        .transform
        .first_xy_column();
    let angle = target_y.atan2(target_x) - y.atan2(x);
    let (width, height) = if angle.abs() > 0.0001 {
        let (sin, cos) = angle.sin_cos();
        let corners = [
            [min[0], min[1]],
            [max[0], min[1]],
            [max[0], max[1]],
            [min[0], max[1]],
        ]
        .map(|[x, y]| Point3d::new(x * cos - y * sin, x * sin + y * cos, 0.0));
        let min_x = corners
            .iter()
            .map(|point| point.x)
            .fold(f64::INFINITY, f64::min);
        let min_y = corners
            .iter()
            .map(|point| point.y)
            .fold(f64::INFINITY, f64::min);
        let max_x = corners
            .iter()
            .map(|point| point.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let max_y = corners
            .iter()
            .map(|point| point.y)
            .fold(f64::NEG_INFINITY, f64::max);
        (max_x - min_x, max_y - min_y)
    } else {
        (max[0] - min[0], max[1] - min[1])
    };
    let width = (width / factor) as i64;
    let height = (height / factor) as i64;
    BoundingBox::new(
        Point::new(-width / 2, -height / 2),
        Point::new(width / 2, height / 2),
    )
}

fn absolute(width: FloatOrPercent, nozzle: f64) -> f64 {
    match width {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(value) => nozzle * value.0 / 100.0,
    }
}
