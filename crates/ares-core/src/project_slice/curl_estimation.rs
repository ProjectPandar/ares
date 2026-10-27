//! Source-owned curled-perimeter estimation stage
//! (`PrintObject::estimate_curled_extrusions`, `PrintObject.cpp:897-913`
//! → `SupportSpotsGenerator::estimate_malformations`,
//! `SupportSpotsGenerator.cpp:141-196`).
//!
//! Runs once per object over the ORIGINAL ordered perimeters — before
//! `simplify_extrusion_path` (`Print.cpp:2303/2393` before `:2571`) and
//! before seam placement (`GCode.cpp:5752-5775` copies and splits the
//! loop only at emit). Each external entity's concatenated sub-path
//! points (`collect_points`) are estimated with the region's nominal
//! `frExternalPerimeter` flow width (`get_flow_width`,
//! `SupportSpotsGenerator.cpp:92`) — not the emitted path width — and
//! the results become the per-layer `Layer::curled_lines` the overhang
//! estimator consumes one layer later.

use crate::{
    ObjectOptions, OrcaFloats, SliceError,
    geometry::{CoordinateScale, LineDistanceTree},
    project_slice::{
        gcode_emit::motion::{CurlEstimate, CurledLine, ExternalEntityPoints, estimate_object},
        island_print_order::{
            IslandPrintEntity, OrderedExtrusionLayer, PreparedPostIslandPrintOrder,
        },
        perimeters::{
            classic::{
                entity_collections::ExtrusionEntity,
                materialize::{ExtrusionPath, ExtrusionRole},
            },
            flow::resolve_nominal_external_perimeter_flow,
            types::{PostPerimeterInputPrintObject, RegionLayerIndex},
        },
    },
};

/// One entry per object: each physical layer's `Layer::curled_lines`.
pub(in crate::project_slice) fn estimate(
    prepared: &PreparedPostIslandPrintOrder,
) -> Result<Vec<Vec<Vec<CurledLine>>>, SliceError> {
    let traversal = &prepared
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let scale = traversal.scale;
    let nozzle_diameters = &traversal.resolved.views.full.project.print.nozzle_diameter;
    let mut objects = Vec::with_capacity(prepared.objects.len());
    for (object_index, layers) in prepared.objects.iter().enumerate() {
        // `PrintObject.cpp:899-901`: any print region enabling overhang
        // speed runs the estimation for the whole object.
        let resolved = traversal.print_object_config(object_index).0;
        if !resolved
            .all_regions
            .iter()
            .any(|region| region.enable_overhang_speed.0)
        {
            objects.push(Vec::new());
            continue;
        }
        let object = &traversal.objects[object_index];
        let input = &object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .object;
        let heights = object.physical_layers();
        let mut widths = NominalWidths::new(input, &resolved.object, nozzle_diameters);
        // The boundary trees borrow the lower layers' merged lines, so
        // both live for the whole object pass (`lower_layer->lslices`).
        let lower_lines = layers
            .iter()
            .enumerate()
            .map(|(layer_index, _)| {
                object.lower_slices(layer_index).map(|slices| {
                    slices
                        .iter()
                        .flat_map(crate::geometry::ExPolygon::lines)
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();
        let boundaries = lower_lines
            .iter()
            .map(|lines| lines.as_ref().map(|lines| LineDistanceTree::new(lines)))
            .collect::<Vec<_>>();
        let mut estimated = Vec::with_capacity(layers.len());
        for (layer_index, layer) in layers.iter().enumerate() {
            let entities = external_entities(layer, &mut widths, scale, layer_index)?;
            estimated.push(CurlEstimate {
                entities,
                height: heights[layer_index].height as f32,
                lower_boundary: boundaries[layer_index].as_ref(),
            });
        }
        objects.push(estimate_object(scale, &estimated));
    }
    Ok(objects)
}

/// Per-region cache of the nominal `frExternalPerimeter` flow width
/// (`get_flow_width`, resolved once per region like upstream's per-layer
/// `get_flow_width` on unchanged region config).
struct NominalWidths<'a> {
    input: &'a PostPerimeterInputPrintObject,
    object: &'a ObjectOptions,
    nozzle_diameters: &'a OrcaFloats,
    widths: Vec<Option<f32>>,
}

impl NominalWidths<'_> {
    fn new<'a>(
        input: &'a PostPerimeterInputPrintObject,
        object: &'a ObjectOptions,
        nozzle_diameters: &'a OrcaFloats,
    ) -> NominalWidths<'a> {
        NominalWidths {
            input,
            object,
            nozzle_diameters,
            widths: Vec::new(),
        }
    }

    fn resolve(&mut self, region_index: usize, layer_index: usize) -> Result<f32, SliceError> {
        if let Some(Some(width)) = self.widths.get(region_index) {
            return Ok(*width);
        }
        let record = self.input.records[self.input.region_layer_record_index(RegionLayerIndex {
            region_index,
            layer_index,
        })]
        .as_ref()
        .expect("a printed perimeter region has an input record");
        let width = resolve_nominal_external_perimeter_flow(
            self.object,
            self.input.region_options(record),
            self.nozzle_diameters,
        )?
        .width;
        if self.widths.len() <= region_index {
            self.widths.resize(region_index + 1, None);
        }
        self.widths[region_index] = Some(width);
        Ok(width)
    }
}

/// The entity's concatenated sub-path polylines when it is an external
/// perimeter (`collect_points`, `ExtrusionEntity.hpp:507-512,431-435`;
/// role = first path's role, `ExtrusionEntity.hpp:415`).
fn external_paths(item: &ExtrusionEntity) -> Option<&[ExtrusionPath]> {
    let paths = match item {
        ExtrusionEntity::Loop(ordered) => &ordered.extrusion_loop.paths,
        ExtrusionEntity::MultiPath(multi_path) => &multi_path.paths,
        ExtrusionEntity::Path(path) => std::slice::from_ref(path),
    };
    paths
        .first()
        .is_some_and(|path| path.role == ExtrusionRole::ExternalPerimeter)
        .then_some(paths)
}

/// The layer's external-perimeter entities in collected order — each
/// island's perimeter collections flattened per entity
/// (`layer_region->perimeters.flatten()`) with the owning region's
/// nominal `frExternalPerimeter` flow width.
fn external_entities(
    layer: &OrderedExtrusionLayer,
    widths: &mut NominalWidths<'_>,
    scale: CoordinateScale,
    layer_index: usize,
) -> Result<Vec<ExternalEntityPoints>, SliceError> {
    let mut entities = Vec::new();
    for island in &layer.islands {
        // `perimeter_regions` is a queue aligned with the Perimeter
        // entities in `entities` order (consumed like
        // `emit_layer`'s `perimeter_ids.next()`), not a positional zip —
        // `infill_first` layers place fills ahead of the walls.
        let mut regions = island.perimeter_regions.iter();
        for entity in &island.entities {
            if let IslandPrintEntity::Perimeter(collection) = entity {
                let &region_index = regions.next().expect("perimeter retains its source region");
                let width = widths.resolve(region_index, layer_index)?;
                push_entity_points(&collection.entities, width, scale, &mut entities);
            }
        }
    }
    Ok(entities)
}

/// Each external entity's concatenated points with its region's nominal
/// width (`collect_points`, `ExtrusionEntity.hpp:507-512,431-435`).
fn push_entity_points(
    items: &[ExtrusionEntity],
    width: f32,
    scale: CoordinateScale,
    entities: &mut Vec<ExternalEntityPoints>,
) {
    for item in items {
        let Some(paths) = external_paths(item) else {
            continue;
        };
        entities.push((
            paths
                .iter()
                .flat_map(|path| path.polyline.points.iter())
                .map(|point| (scale.unscale(point.x), scale.unscale(point.y)))
                .collect::<Vec<_>>(),
            width,
        ));
    }
}
