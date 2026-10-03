//! Native LayerRegion support views and PrintRegion::bridging_height_avg.

use crate::{
    ExtrusionRole, ObjectOptions, OrcaFloats, RegionOptions, SliceError,
    geometry::{CoordinateScale, ExPolygon, Point, Polyline},
    project_slice::{
        fill_entities::{FillExtrusionEntity, LayerFillEntities},
        layers::PlannedLayer,
        perimeters::{
            classic::{
                entity_collections::{ExtrusionEntity, ExtrusionEntityCollection},
                materialize::{ExtrusionPath, ExtrusionRole as PerimeterRole},
            },
            flow::resolve_perimeter_flows,
            types::PerimeterFlows,
        },
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::RegionSurfaceKind,
        support_material::{
            layer_input::RegionInput, remove_bridges::RegionBridges, trim_object::TrimRegion,
        },
    },
};

pub(super) struct Region<'a> {
    pub(super) options: &'a RegionOptions,
    pub(super) raw_slices: &'a [ExPolygon],
    pub(super) slices: Vec<ExPolygon>,
    pub(super) tops: Vec<ExPolygon>,
    pub(super) bottoms: Vec<ExPolygon>,
    bottom_bridges: Vec<ExPolygon>,
    known_bottom_bridges: Vec<ExPolygon>,
    known_bottom_bridge_angles: Vec<f64>,
    perimeter_lines: Vec<Polyline>,
    perimeters: &'a [ExtrusionEntityCollection],
    fills: &'a LayerFillEntities,
    region_index: usize,
    flows: PerimeterFlows,
    pub(super) bridging_height: f64,
    outer_nozzle: f64,
}

pub(super) struct RegionSource<'a> {
    pub(super) region_index: usize,
    pub(super) options: &'a RegionOptions,
    pub(super) object: &'a ObjectOptions,
    pub(super) layer: &'a PlannedLayer,
    pub(super) raw_slices: &'a [ExPolygon],
    pub(super) typed: Option<&'a PreparedSurfaceTypeRecord>,
    pub(super) fills: &'a LayerFillEntities,
}

impl<'a> Region<'a> {
    pub(super) fn new(
        source: RegionSource<'a>,
        settings: &crate::ProjectSettings,
    ) -> Result<Self, SliceError> {
        let nozzles = &settings.project.print.nozzle_diameter;
        let flows = resolve_perimeter_flows(
            source.layer,
            settings.process.print.initial_layer_line_width,
            source.options,
            source.object,
            nozzles,
        )?;
        let slices = source.typed.into_iter().flat_map(|record| &record.slices);
        let fills = source
            .typed
            .into_iter()
            .flat_map(|record| &record.fill_surfaces);
        let bottom_bridges = fills
            .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::BottomBridge)
            .collect::<Vec<_>>();
        let perimeters = source.fills.region_perimeters(source.region_index);
        let perimeter_lines = perimeters
            .iter()
            .flat_map(|collection| &collection.entities)
            .map(|entity| as_polyline(paths(entity)))
            .collect();
        Ok(Self {
            options: source.options,
            raw_slices: source.raw_slices,
            slices: slices
                .clone()
                .map(|surface| surface.as_parts().1.clone())
                .collect(),
            tops: slices
                .clone()
                .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Top)
                .map(|surface| surface.as_parts().1.clone())
                .collect(),
            bottoms: slices
                .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::Bottom)
                .map(|surface| surface.as_parts().1.clone())
                .collect(),
            known_bottom_bridges: bottom_bridges
                .iter()
                .filter(|surface| surface.as_parts().4 >= 0.0)
                .map(|surface| surface.as_parts().1.clone())
                .collect(),
            known_bottom_bridge_angles: bottom_bridges
                .iter()
                .filter(|surface| surface.as_parts().4 >= 0.0)
                .map(|surface| surface.as_parts().4)
                .collect(),
            bottom_bridges: bottom_bridges
                .into_iter()
                .map(|surface| surface.as_parts().1.clone())
                .collect(),
            perimeter_lines,
            perimeters,
            fills: source.fills,
            region_index: source.region_index,
            flows,
            bridging_height: bridging_height(source.options, nozzles),
            outer_nozzle: nozzle(source.options.outer_wall_filament_id.0, nozzles),
        })
    }

    pub(super) fn view(&self, scale: CoordinateScale) -> RegionInput<'_> {
        RegionInput {
            slices: &self.slices,
            raw_slices: self.raw_slices,
            external_width_scaled: (f64::from(self.flows.ext_perimeter_flow.width) / scale.factor())
                as i64 as f32,
            bridges: RegionBridges {
                perimeters: &self.perimeter_lines,
                outer_nozzle_scaled: (self.outer_nozzle / scale.factor()) as f32,
                bridge_width_scaled: (f64::from(self.flows.overhang_flow.width) / scale.factor())
                    as i64 as f64,
                bridge_spacing_scaled: (f64::from(self.flows.overhang_flow.spacing)
                    / scale.factor()) as i64 as f64,
                bottom_bridge_surfaces: &self.known_bottom_bridges,
                // Only the disabled LayerRegion.cpp #else branch populates these.
                unsupported_edges: &[],
            },
        }
    }

    pub(super) fn trim(&self) -> TrimRegion<'_> {
        TrimRegion {
            bridging_height: self.bridging_height,
            bottom_bridges: &self.bottom_bridges,
            detect_overhang_wall: self.options.detect_overhang_wall.0,
            perimeters: self.perimeters,
        }
    }

    pub(super) fn tree_bottom_bridges(
        &self,
    ) -> Vec<crate::project_slice::support_material::tree::remove_bridges::BottomBridge<'_>> {
        self.known_bottom_bridges
            .iter()
            .zip(&self.known_bottom_bridge_angles)
            .map(|(polygon, &angle)| {
                crate::project_slice::support_material::tree::remove_bridges::BottomBridge {
                    polygon,
                    angle,
                }
            })
            .collect()
    }

    pub(super) fn has_bridging(&self) -> bool {
        self.perimeters
            .iter()
            .flat_map(|collection| &collection.entities)
            .any(has_bridging_loop)
            || (!self.bottom_bridges.is_empty()
                && self
                    .fills
                    .region_collections(self.region_index)
                    .flat_map(|collection| &collection.entities)
                    .any(|entity| match entity {
                        FillExtrusionEntity::Path(path) | FillExtrusionEntity::Oriented(path) => {
                            matches!(
                                path.role,
                                ExtrusionRole::BridgeInfill | ExtrusionRole::InternalBridgeInfill
                            )
                        }
                        FillExtrusionEntity::VariableWidth(_) => false,
                    }))
    }
}

fn has_bridging_loop(entity: &ExtrusionEntity) -> bool {
    let ExtrusionEntity::Loop(loop_) = entity else {
        return false;
    };
    loop_
        .extrusion_loop
        .paths
        .iter()
        .find(|path| {
            path.role == PerimeterRole::OverhangPerimeter && !path.polyline.points.is_empty()
        })
        .is_some_and(|path| {
            let points = &path.polyline.points;
            points.len()
                >= if points.first() == points.last() {
                    3
                } else {
                    2
                }
        })
}

fn paths(entity: &ExtrusionEntity) -> &[ExtrusionPath] {
    match entity {
        ExtrusionEntity::Loop(loop_) => &loop_.extrusion_loop.paths,
        ExtrusionEntity::MultiPath(multi) => &multi.paths,
        ExtrusionEntity::Path(path) => std::slice::from_ref(path),
    }
}

fn as_polyline(paths: &[ExtrusionPath]) -> Polyline {
    let mut points = Vec::new();
    if let Some(first) = paths.first() {
        let point = first.polyline.points[0];
        points.push(Point::new(point.x, point.y));
    }
    points.extend(
        paths
            .iter()
            .flat_map(|path| &path.polyline.points[1..])
            .map(|point| Point::new(point.x, point.y)),
    );
    Polyline::new(points)
}

pub(in crate::project_slice::support_material) fn nozzle(
    selector: i32,
    nozzles: &OrcaFloats,
) -> f64 {
    nozzles
        .0
        .get((selector - 1) as usize)
        .unwrap_or(&nozzles.0[0])
        .0
}

fn bridging_height(region: &RegionOptions, nozzles: &OrcaFloats) -> f64 {
    [
        region.outer_wall_filament_id,
        region.inner_wall_filament_id,
        region.sparse_infill_filament_id,
        region.internal_solid_filament_id,
        region.top_surface_filament_id,
        region.bottom_surface_filament_id,
    ]
    .into_iter()
    .map(|selector| nozzle(selector.0, nozzles))
    .sum::<f64>()
        / 6.0
        * region.bridge_flow.0.sqrt()
}
