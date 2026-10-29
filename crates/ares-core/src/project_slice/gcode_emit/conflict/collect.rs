//! `ConflictChecker.cpp::getAllLayersExtrusionPathsFromObject` and entity traversal.
use super::Layer;
use crate::{
    ExtrusionRole,
    geometry::Point,
    project_slice::{
        fill_entities::FillExtrusionEntity,
        island_print_order::{IslandPrintEntity, OrderedExtrusionLayer},
        perimeters::classic::{
            entity_collections::ExtrusionEntity, gap_extrusion::GapFillEntity,
            materialize::Polyline3, traversal::PostClassicTraversalPrintObject,
        },
        raft::RaftLayerPlan,
        support_material::InstalledSupportLayer,
    },
};

pub(super) fn object(
    layers: &[OrderedExtrusionLayer],
    object: &PostClassicTraversalPrintObject,
) -> Vec<Layer> {
    layers
        .iter()
        .zip(object.physical_layers())
        .map(|(layer, physical)| {
            let mut output = Layer::new((physical.print_z - physical.height) as f32);
            ordered(&mut output, layer);
            output
        })
        .collect()
}

pub(super) fn support(
    layers: &[InstalledSupportLayer],
    raft: Option<(&[RaftLayerPlan], &[OrderedExtrusionLayer])>,
) -> Vec<Layer> {
    let mut output = Vec::new();
    if let Some((plans, layers)) = raft {
        for (plan, layer) in plans.iter().zip(layers) {
            let mut target = Layer::new((plan.z.print_z - plan.height_mm) as f32);
            ordered(&mut target, layer);
            output.push(target);
        }
    }
    for layer in layers {
        let mut target = Layer::new((layer.print_z - layer.height) as f32);
        for entity in &layer.support_fills {
            entity.for_each_path(&mut |path| {
                polyline3(&mut target, &path.polyline, is_support(path.role))
            });
        }
        output.push(target);
    }
    output.sort_by(|a, b| a.bottom.total_cmp(&b.bottom));
    output
}

fn ordered(output: &mut Layer, layer: &OrderedExtrusionLayer) {
    for entity in layer.islands.iter().flat_map(|island| &island.entities) {
        match entity {
            IslandPrintEntity::Perimeter(collection) => {
                for entity in &collection.entities {
                    perimeter(output, entity);
                }
            }
            IslandPrintEntity::Fill(path) => fill(output, path),
            IslandPrintEntity::FillCollection(collection) => {
                for path in &collection.entities {
                    fill(output, path);
                }
            }
            IslandPrintEntity::Thin(path) => gap(output, path),
        }
    }
}

fn perimeter(output: &mut Layer, entity: &ExtrusionEntity) {
    match entity {
        ExtrusionEntity::Loop(loop_) => {
            for path in &loop_.extrusion_loop.paths {
                polyline3(output, &path.polyline, false);
            }
        }
        ExtrusionEntity::MultiPath(multi) => {
            for path in &multi.paths {
                polyline3(output, &path.polyline, false);
            }
        }
        ExtrusionEntity::Path(path) => polyline3(output, &path.polyline, false),
    }
}

fn fill(output: &mut Layer, entity: &FillExtrusionEntity) {
    match entity {
        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => output.path(
            path.polyline
                .points()
                .iter()
                .map(|point| crate::geometry::Point::new(point.x, point.y)),
            is_support(path.role),
        ),
        FillExtrusionEntity::VariableWidth(path) => gap(output, path),
    }
}
fn gap(output: &mut Layer, entity: &GapFillEntity) {
    match entity {
        GapFillEntity::Path(path) => polyline3(output, &path.polyline, false),
        GapFillEntity::Loop(paths) => {
            for path in paths {
                polyline3(output, &path.polyline, false);
            }
        }
    }
}
fn polyline3(output: &mut Layer, polyline: &Polyline3, support: bool) {
    output.path(
        polyline
            .points
            .iter()
            .map(|point| Point::new(point.x, point.y)),
        support,
    );
}
fn is_support(role: ExtrusionRole) -> bool {
    matches!(
        role,
        ExtrusionRole::SupportMaterial
            | ExtrusionRole::SupportMaterialInterface
            | ExtrusionRole::SupportTransition
    )
}
