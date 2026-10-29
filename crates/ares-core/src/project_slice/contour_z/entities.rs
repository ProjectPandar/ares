//! ContourZ.cpp:162-243: eligible perimeter and fill entity traversal.
use super::{
    mesh::IndexedMesh,
    path::{self, Context},
};
use crate::project_slice::perimeters::classic::{
    entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
    materialize::{ExtrusionPath, ExtrusionRole},
};
pub(super) fn collection(
    collection: &mut ExtrusionEntityCollection,
    mesh: &IndexedMesh,
    context: &Context<'_>,
) {
    for entity in &mut collection.entities {
        match entity {
            ExtrusionEntity::Loop(loop_) => {
                for item in &mut loop_.extrusion_loop.paths {
                    apply(item, mesh, context);
                }
            }
            ExtrusionEntity::MultiPath(multi) => {
                for item in &mut multi.paths {
                    apply(item, mesh, context);
                }
            }
            ExtrusionEntity::Path(item) => apply(item, mesh, context),
        }
    }
}
fn apply(path: &mut ExtrusionPath, mesh: &IndexedMesh, context: &Context<'_>) {
    let role = match path.role {
        ExtrusionRole::ExternalPerimeter => crate::ExtrusionRole::ExternalPerimeter,
        ExtrusionRole::Perimeter => crate::ExtrusionRole::Perimeter,
        ExtrusionRole::TopSolidInfill => crate::ExtrusionRole::TopSolidInfill,
        ExtrusionRole::OverhangPerimeter
        | ExtrusionRole::GapFill
        | ExtrusionRole::SolidInfill
        | ExtrusionRole::BottomSurface => return,
    };
    if let Some(points) = path::contour(&path.polyline.points, role, path.width, mesh, context) {
        path.polyline.points = points;
        path.polyline.fitting.clear();
        path.z_contoured = true;
    }
}

pub(super) fn prepare_layer(
    layer: &mut crate::project_slice::fill_entities::LayerFillEntities,
    planned: &crate::project_slice::layers::PlannedLayer,
    regions: &[crate::project_slice::region_slices::PostRegion],
    mesh: &IndexedMesh,
    scale: crate::geometry::CoordinateScale,
) {
    for (region_index, region) in regions.iter().enumerate() {
        let options = region.as_parts().1;
        if !options.zaa_enabled.0 {
            continue;
        }
        let context = Context {
            region: options,
            slice_z: planned.slice_z,
            print_z: planned.print_z,
            layer_height: planned.height,
            scale,
        };
        for (_, range) in layer
            .collection_ranges
            .iter()
            .filter(|(owner, _)| *owner == region_index)
        {
            for entity in layer.collections[range.clone()]
                .iter_mut()
                .flat_map(|collection| &mut collection.entities)
            {
                fill(entity, mesh, &context);
            }
        }
        for item in &mut layer.perimeters[layer.perimeter_ranges[region_index].clone()] {
            collection(item, mesh, &context);
        }
    }
}

fn fill(
    entity: &mut crate::project_slice::fill_entities::FillExtrusionEntity,
    mesh: &IndexedMesh,
    context: &Context<'_>,
) {
    use crate::project_slice::{
        fill_entities::FillExtrusionEntity, perimeters::classic::gap_extrusion::GapFillEntity,
    };
    match entity {
        FillExtrusionEntity::Path(item) | FillExtrusionEntity::Oriented(item) => {
            if let Some(points) =
                path::contour(&item.polyline.points, item.role, item.width, mesh, context)
            {
                item.polyline.points = points;
                item.polyline.fitting.clear();
                item.z_contoured = true;
            }
        }
        FillExtrusionEntity::VariableWidth(GapFillEntity::Path(item)) => apply(item, mesh, context),
        FillExtrusionEntity::VariableWidth(GapFillEntity::Loop(paths)) => {
            for item in paths {
                apply(item, mesh, context);
            }
        }
    }
}
