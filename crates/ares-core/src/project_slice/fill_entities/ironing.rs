use crate::fill::plane_path::PlanePathPattern;
use crate::project_slice::perimeters::classic::materialize::Polyline3;
// Source boundary: OrcaSlicer 2.4.2 `Fill/Fill.cpp::Layer::make_ironing`.

use crate::{
    ExtrusionRole, ObjectOptions, OrcaFloats, OrcaInts, ProcessInfillPattern,
    ProcessRegionSourceOptions, RegionOptions, SliceError,
    fill::rectilinear::{MonotonicFillParams, fill_monotonic_surface},
    geometry::{CoordinateScale, ExPolygon, JoinType, Point, offset_expolygon},
    project_slice::{
        layers::PlannedLayer,
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
    },
};

use super::{
    FillExtrusionCollection, FillExtrusionEntity, FillExtrusionPath, LayerFillEntities,
    geometry_error,
};

mod areas;
mod plane_path;
mod spans;
mod surface;
pub(super) use spans::{SpanGeneration, append_spans};

const ROUNDED_RECTANGLE_FACTOR: f64 = 1.0 - 0.25 * std::f64::consts::PI;

pub(super) struct Input<'a> {
    pub(super) record: &'a PreparedSurfaceTypeRecord,
    pub(super) layer_slices: &'a [ExPolygon],
    pub(super) region: &'a RegionOptions,
    pub(super) dont_alternate_fill_direction: bool,
    pub(super) object: &'a ObjectOptions,
    pub(super) nozzles: &'a OrcaFloats,
    pub(super) filament_map: &'a OrcaInts,
    pub(super) is_bbl: bool,
    pub(super) layer: &'a PlannedLayer,
    pub(super) last_layer: bool,
    pub(super) spiral_mode: bool,
    pub(super) scale: CoordinateScale,
}

pub(super) struct SpanContext<'a> {
    pub(super) object: &'a ObjectOptions,
    pub(super) default_region: &'a ProcessRegionSourceOptions,
    pub(super) layer: &'a PlannedLayer,
    pub(super) layers: &'a [PlannedLayer],
    pub(super) model_rotation_offset: f64,
}

#[derive(PartialEq)]
pub(super) struct SpanSortKey {
    extruder: i32,
    spacing: f64,
    height: f64,
    speed: f64,
    angle: f64,
    fixed_angle: bool,
    inset: f64,
    pattern: ProcessInfillPattern,
}

impl SpanSortKey {
    pub(super) fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.extruder
            .cmp(&other.extruder)
            .then_with(|| self.spacing.partial_cmp(&other.spacing).unwrap())
            .then_with(|| self.height.partial_cmp(&other.height).unwrap())
            .then_with(|| self.speed.partial_cmp(&other.speed).unwrap())
            .then_with(|| self.angle.partial_cmp(&other.angle).unwrap())
            .then_with(|| self.fixed_angle.cmp(&other.fixed_angle))
            .then_with(|| self.inset.partial_cmp(&other.inset).unwrap())
    }
}

pub(super) fn span_sort_key(region: &RegionOptions, context: &SpanContext<'_>) -> SpanSortKey {
    let spacing = region.filament_ironing_spacing.0;
    let height = context.object.layer_height.0 * 0.01 * region.filament_ironing_flow.0;
    let (angle, fixed_angle) = areas::ironing_angle(region, context);
    SpanSortKey {
        extruder: region.top_surface_filament_id.0,
        spacing,
        height,
        speed: region.filament_ironing_speed.0,
        angle,
        fixed_angle,
        inset: region.filament_ironing_inset.0,
        pattern: region.ironing_pattern,
    }
}

pub(super) fn append_areas(
    output: &mut LayerFillEntities,
    input: &Input<'_>,
    ironing_areas: Vec<ExPolygon>,
    (angle, fixed_angle): (f64, bool),
) -> Result<(), SliceError> {
    let spacing = input.region.filament_ironing_spacing.0;
    if spacing <= 0.0 {
        return Ok(());
    }
    let nozzle = areas::selected_nozzle(input);
    let angle = angle as f32;
    let extrusion_height =
        input.object.layer_height.0 * 0.01 * input.region.filament_ironing_flow.0 * spacing
            / nozzle;
    let height = extrusion_height as f32;
    let width = (f64::from(nozzle as f32) + f64::from(height) * ROUNDED_RECTANGLE_FACTOR) as f32;
    let mm3_per_mm = nozzle * extrusion_height;

    for area in ironing_areas {
        let polylines = match input.region.ironing_pattern {
            // Fill.cpp:1634 creates FillConcentric for closed-loop ironing.
            ProcessInfillPattern::Concentric => {
                // `Fill::fill_surface` contracts the surface by
                // `scale_(overlap - 0.5*spacing)` before
                // `_fill_surface_single` (`FillBase.cpp:107`); ironing sets
                // `f->overlap = 0` (`Fill.cpp:1631`), so the ring domain is
                // inset by half the ironing spacing. This is where upstream's
                // concentric-ironing outer ring lands at spacing/2 inside the
                // clipped ironing area.
                let domain_offset = (-0.5 * spacing / input.scale.factor()) as f32;
                let generation_domains =
                    offset_expolygon(&area, domain_offset, JoinType::Miter, 3.0)
                        .map_err(geometry_error)?;
                let mut polylines = Vec::new();
                for domain in generation_domains {
                    polylines.extend(
                        crate::fill::concentric_sparse::fill_concentric_sparse(
                            &domain,
                            spacing,
                            &crate::fill::FillParams {
                                density: 1.0,
                                multiline: 1,
                                dont_adjust: true,
                            },
                            0,
                            input.scale,
                        )
                        .map_err(geometry_error)?,
                    );
                }
                if polylines.is_empty() {
                    continue;
                }
                output.collections.push(FillExtrusionCollection {
                    entities: polylines
                        .into_iter()
                        .map(|polyline| {
                            FillExtrusionEntity::Path(FillExtrusionPath {
                                polyline: Polyline3::from_xy(polyline, Vec::new()),
                                z_contoured: false,
                                role: ExtrusionRole::Ironing,
                                mm3_per_mm,
                                width,
                                height,
                            })
                        })
                        .collect(),
                    no_sort: true,
                });
                continue;
            }
            ProcessInfillPattern::HilbertCurve => plane_path::polylines(
                &area,
                PlanePathPattern::HilbertCurve,
                spacing,
                angle,
                input.scale,
            )?,
            ProcessInfillPattern::ArchimedeanChords => plane_path::polylines(
                &area,
                PlanePathPattern::ArchimedeanChords,
                spacing,
                angle,
                input.scale,
            )?,
            ProcessInfillPattern::OctagramSpiral => plane_path::polylines(
                &area,
                PlanePathPattern::OctagramSpiral,
                spacing,
                angle,
                input.scale,
            )?,
            // Zag classes inherit this Polylines path (FillRectilinear.hpp:193-224).
            // Ironing is monotonic (Fill.cpp:1619-1621): FillRectilinear.cpp:2874
            // is not reached; LockedZag's extrusion override is not called.
            // `AlignedRectilinear` shares this arm because `FillAlignedRectilinear`
            // derives from `FillRectilinear` and therefore consumes the same
            // `fill_params.monotonic = true` (FillRectilinear.cpp:2853-2867); its
            // only deviation is `_layer_angle() == 0` (FillRectilinear.hpp:38-46),
            // carried by `layer_angle_fixed`.
            ProcessInfillPattern::Rectilinear
            | ProcessInfillPattern::AlignedRectilinear
            | ProcessInfillPattern::ZigZag
            | ProcessInfillPattern::CrossZag
            | ProcessInfillPattern::LockedZag => {
                fill_monotonic_surface(
                    &area,
                    MonotonicFillParams {
                        // Fill.cpp:1631/1705-1723 -> FillBase.cpp:311-313.
                        dont_alternate_fill_direction: input.dont_alternate_fill_direction,
                        spacing,
                        overlap: 0.0,
                        density: 1.0,
                        angle,
                        layer_index: input.layer.id,
                        thickness_layers: 1,
                        fixed_angle,
                        // `FillAlignedRectilinear::_layer_angle()` returns 0.f
                        // (`FillRectilinear.hpp:38-46`), so aligned ironing never
                        // alternates by layer parity; every other ironing pattern
                        // reaching this arm inherits the `FillBase.hpp:211` default.
                        layer_angle_fixed: input.region.ironing_pattern
                            == ProcessInfillPattern::AlignedRectilinear,
                        bridge_angle: None,
                        reference_point: Point::new(0, 0),
                        dont_adjust: true,
                        anchor_length_max: 1000.0,
                        link_max_length: 3.0 * spacing,
                        // Ironing bypasses the consistent-pattern traversal
                        // (FillRectilinear.cpp:2874).
                        consistent_pattern: false,
                        horiz_move: 0.0,
                        extended_object_min_x: None,
                        extended_object_max_x: None,
                        locked_zag: false,
                    },
                    input.scale,
                )
                .map_err(geometry_error)?
                .polylines
            }
            // `FillAdaptive::Filler`/`FillLightning::Filler::_fill_surface_single`
            // SIGSEGV upstream inside `make_ironing` — the fills need a
            // precomputed octree/generator context `Layer::make_ironing`
            // (`Fill.cpp:1573-1705`) never installs (probe:
            // `ironing-pattern-probe/native-classification.txt`). Keep the
            // bounded `UnsupportedProjectFeature` guard per
            // `docs/architecture/ironing-pattern-coverage-spec.md`.
            // `Fill3DHoneycomb` has no defined result here: `make_ironing` leaves
            // `layer_height` at zero, so its Z scale is non-finite
            // (`Fill3DHoneycomb.cpp:371-391`).
            ProcessInfillPattern::AdaptiveCubic
            | ProcessInfillPattern::SupportCubic
            | ProcessInfillPattern::Lightning
            | ProcessInfillPattern::ThreeDHoneycomb => {
                return Err(SliceError::UnsupportedProjectFeature(
                    "ironing_pattern".to_owned(),
                ));
            }
            // All remaining `ProcessInfillPattern` variants are handled
            // upstream by `Fill::new_from_type` (`FillBase.cpp:40-77`) via the
            // shared surface-fill dispatcher — see
            // `ironing::surface::polylines` for the `Fill.cpp:1631-1705`
            // ironing `FillParams` wiring.
            pattern => surface::polylines(&surface::IroningFillInput {
                pattern,
                area: &area,
                spacing,
                angle,
                fixed_angle,
                dont_alternate_fill_direction: input.dont_alternate_fill_direction,
                layer_id: input.layer.id,
                print_z: input.layer.print_z,
                layer_height: input.object.layer_height.0,
                scale: input.scale,
                flow_width: width,
                flow_height: height,
                flow_mm3_per_mm: mm3_per_mm,
                nozzle_diameter: nozzle as f32,
            })?,
        };
        if polylines.is_empty() {
            continue;
        }
        output.collections.push(FillExtrusionCollection {
            entities: polylines
                .into_iter()
                .map(|polyline| {
                    FillExtrusionEntity::Path(FillExtrusionPath {
                        polyline: Polyline3::from_xy(polyline, Vec::new()),
                        z_contoured: false,
                        role: ExtrusionRole::Ironing,
                        mm3_per_mm,
                        width,
                        height,
                    })
                })
                .collect(),
            no_sort: true,
        });
    }
    Ok(())
}
