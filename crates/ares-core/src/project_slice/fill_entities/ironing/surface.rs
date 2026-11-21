//! `Fill.cpp:1573-1705` ironing-arm surface dispatch: every
//! `ProcessInfillPattern` variant not already handled by a dedicated
//! upstream `Layer::make_ironing` specialization goes through the shared
//! `fill_entities::surface` dispatcher — the same `Fill::new_from_type`
//! (`FillBase.cpp:40-77`) entry `support_ironing` calls
//! (`SupportCommon.cpp:1879`). The crash-class patterns (`AdaptiveCubic`,
//! `SupportCubic`, `Lightning`) upstream SIGSEGV inside `make_ironing` and
//! `ThreeDHoneycomb` has no defined result there — there is no oracle output to
//! mirror; they keep the
//! bounded `UnsupportedProjectFeature` guard per
//! `docs/architecture/ironing-pattern-coverage-spec.md`.

use crate::{
    ExtrusionRole, ProcessInfillPattern, SliceError,
    geometry::{ExPolygon, Point},
    project_slice::{
        group_fills::{SurfaceFillParams, SurfaceFillPattern},
        perimeters::types::Flow,
    },
};

/// `Fill.cpp:1631-1705` ironing `FillParams`: `density=1`, `monotonic`,
/// `overlap=0`, `link_max_length=3*spacing`, `dont_adjust`; the wrapper
/// propagates `dont_alternate_fill_direction` (Fill.cpp:1617-1628).
pub(super) struct IroningFillInput<'a> {
    pub(super) pattern: ProcessInfillPattern,
    pub(super) area: &'a ExPolygon,
    pub(super) spacing: f64,
    pub(super) angle: f32,
    pub(super) fixed_angle: bool,
    pub(super) dont_alternate_fill_direction: bool,
    pub(super) layer_id: usize,
    pub(super) print_z: f64,
    pub(super) layer_height: f64,
    pub(super) scale: crate::geometry::CoordinateScale,
    pub(super) flow_width: f32,
    pub(super) flow_height: f32,
    pub(super) flow_mm3_per_mm: f64,
    pub(super) nozzle_diameter: f32,
}

pub(super) fn polylines(
    input: &IroningFillInput<'_>,
) -> Result<Vec<crate::geometry::Polyline>, SliceError> {
    let params = SurfaceFillParams {
        extruder: 0,
        pattern: SurfaceFillPattern::Configured(input.pattern),
        spacing: input.spacing,
        overlap: 0.0,
        angle: input.angle,
        fixed_angle: input.fixed_angle,
        bridge: false,
        bridge_angle: 0.0,
        density: 100.0,
        multiline: 1,
        adaptive_line_spacing: 0.0,
        anchor_length: 1000.0,
        anchor_length_max: 1000.0,
        // `SurfaceFillParams::flow` is unused inside `surface_polylines` —
        // only the produced `Polyline` geometry is consumed; role/width/
        // height are fixed by the caller for `ExtrusionRole::Ironing`.
        flow: Flow {
            width: input.flow_width,
            height: input.flow_height,
            spacing: input.spacing as f32,
            nozzle_diameter: input.nozzle_diameter,
            bridge: false,
            mm3_per_mm: input.flow_mm3_per_mm,
        },
        flow_ratio: 1.0,
        extrusion_role: ExtrusionRole::Ironing,
        loop_clipping: 0,
        role_speed: 0.0,
        lateral_lattice_angle_1: -45.0,
        lateral_lattice_angle_2: 45.0,
        infill_lock_depth: 1.0,
        skin_infill_depth: 2.0,
        symmetric_infill_y_axis: false,
        horiz_move: 0.0,
        infill_overhang_angle: 60.0,
        gyroid_optimized: false,
        infill_complete_top: false,
        smooth_factor: 0.0,
        resolution: 0.0,
        filter_out_gap_fill: 0.0,
        gap_fill_target: crate::ProcessGapFillTarget::default(),
        fill_order: crate::ProcessSurfaceFillOrder::Default,
    };
    let call = super::super::types::SurfaceCall {
        dont_alternate_fill_direction: input.dont_alternate_fill_direction,
        layer_id: input.layer_id,
        reference: Point::new(0, 0),
        scale: input.scale,
        link_max_length: 3.0 * input.spacing,
        dont_adjust: true,
    };
    let request = super::super::surface::SurfaceRequest {
        pattern: input.pattern,
        expolygon: input.area.clone(),
        params,
        thickness_layers: 1,
        layer_height: input.layer_height,
        print_z: input.print_z,
        call,
    };
    super::super::surface::surface_polylines(&request)
        .map_err(|error| SliceError::InvalidInput(format!("ironing_pattern fill: {error:?}")))
}
