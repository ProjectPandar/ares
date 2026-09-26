//! Pipeline bridge: derives the full raft construction inputs from
//! `SliceOptions` and the first object layer contours, using the
//! `Slicing.cpp` formulas already ported in `parameters.rs` (raft
//! split/heights/gap via a minimal `SlicingParameters`), then builds
//! the layer plans (`emit.rs`) ready for `apply_raft_to_stream`.

use super::emit::{RaftLayerPlan, raft_layer_plans};
use super::fill_params::RaftFillInputs;
use super::grid::raft_layer_grid;
use super::polygons::{RaftPolygonParams, raft_polygons};
use crate::contours::Contour;
use crate::geometry::support_grid::SupportGridParams;
use crate::geometry::{ClipperError, CoordinateScale, ExPolygon, Point, Polygon};
use crate::project_slice::parameters::SlicingParameters;
use crate::{SliceError, SliceOptions};

const MICRONS_PER_MM: f64 = 1000.0;
const SUPPORT_INTERFACE_SPACING_DEFAULT_MM: f64 = 0.2;
const SUPPORT_BASE_PATTERN_SPACING_DEFAULT_MM: f64 = 0.2;

pub(crate) struct RaftStream {
    pub(crate) plans: Vec<RaftLayerPlan>,
    /// Object layer z shift (`Slicing.cpp:232-236`).
    pub(crate) object_print_z_min: f64,
    #[expect(dead_code, reason = "consumed by the skirt wiring slice")]
    pub(crate) support_flow_spacing_mm: f64,
    #[expect(dead_code, reason = "consumed by the skirt wiring slice")]
    pub(crate) first_layer_flow_spacing_mm: f64,
}

pub(crate) fn build_raft_stream(
    options: &SliceOptions,
    first_layer_contours: &[Contour],
    scale: CoordinateScale,
) -> Result<Option<RaftStream>, SliceError> {
    let raft_layers = options.raft_options()?.layers() as usize;
    if raft_layers == 0 {
        return Ok(None);
    }

    let layer_height = options.layer_height()?;
    let first_height = options.initial_layer_print_height()?;
    let nozzle = options.nozzle_diameters()?.first().copied().unwrap_or(0.4);
    let interface_layers = (raft_layers + 1) / 2;
    let base_layers = raft_layers - interface_layers;
    let heights = f64::max(layer_height, 0.75 * nozzle);

    // `Slicing.cpp:153-160`: `gap_raft_object` is the raw
    // `raft_contact_distance`, rounded to a layer_height multiple ONLY
    // when `independent_support_layer_height` is off. With independent
    // heights the gap stays the raw distance.
    // `Slicing.cpp:153-160` (parallel to `slicing_parameters`): the raw
    // `raft_contact_distance`, rounded only when `independent_support_`
    // `layer_height` is off. This SliceOptions path reads the raw value.
    let independent_support_layer_height = options
        .values()
        .get("independent_support_layer_height")
        .map(|value| {
            value
                .as_bool()
                .unwrap_or_else(|| matches!(value.as_str(), Some("1") | Some("true")))
        })
        .unwrap_or(false);
    let gap_raft_object = if independent_support_layer_height {
        raft_contact_distance(options)
    } else {
        (raft_contact_distance(options) / layer_height + f64::EPSILON).round() * layer_height
    };

    let parameters = SlicingParameters {
        base_raft_layers: base_layers,
        interface_raft_layers: interface_layers,
        base_raft_layer_height: heights,
        interface_raft_layer_height: heights,
        contact_raft_layer_height: heights,
        layer_height,
        min_layer_height: 0.07,
        max_layer_height: 0.28,
        first_print_layer_height: first_height,
        first_object_layer_height: layer_height,
        first_object_layer_bridging: false,
        gap_raft_object,
        gap_object_support: 0.2,
        gap_support_object: 0.2,
        raft_base_top_z: first_height + (base_layers.max(1) - 1) as f64 * heights,
        raft_interface_top_z: first_height
            + (base_layers.max(1) - 1) as f64 * heights
            + (interface_layers.max(1) - 1) as f64 * heights,
        raft_contact_top_z: first_height
            + (base_layers.max(1) - 1) as f64 * heights
            + (interface_layers.max(1) - 1) as f64 * heights
            + heights,
        object_print_z_min: first_height
            + (base_layers.max(1) - 1) as f64 * heights
            + (interface_layers.max(1) - 1) as f64 * heights
            + heights
            + gap_raft_object,
        object_print_z_max: 0.0,
        object_print_z_uncompensated_max: 0.0,
        object_shrinkage_compensation_z: 1.0,
    };

    // First object layer lslices from the stitched contours.
    let scaled_points = |points: &[crate::Point2]| -> Result<Vec<Point>, SliceError> {
        points
            .iter()
            .map(|point| {
                let x = scale
                    .checked_scale(point.x())
                    .ok_or(SliceError::InvalidInput(
                        "raft contour coordinate out of range".to_owned(),
                    ))?;
                let y = scale
                    .checked_scale(point.y())
                    .ok_or(SliceError::InvalidInput(
                        "raft contour coordinate out of range".to_owned(),
                    ))?;
                Ok(Point::new(x, y))
            })
            .collect()
    };
    let lslices = first_layer_contours
        .iter()
        .map(|contour| {
            Ok(ExPolygon::new(
                Polygon::new(scaled_points(contour.points())?),
                Vec::new(),
            ))
        })
        .collect::<Result<Vec<_>, SliceError>>()?;

    // Flow spacings (`Flow::spacing` = width − height·(1 − π/4)).
    let extrusion_options = options.extrusion_options()?;
    if std::env::var("ARES_RAFT_DEBUG").is_ok() {
        eprintln!(
            "RAFTRAW support_line_width={:?} line_width={:?} initial={:?}",
            options.values().get("support_line_width"),
            options.values().get("line_width"),
            options.values().get("initial_layer_line_width")
        );
    }
    let support_width =
        extrusion_options.width_for_role(crate::print_paths::PrintPathRole::SupportMaterial);
    let interface_width = extrusion_options
        .width_for_role(crate::print_paths::PrintPathRole::SupportMaterialInterface);
    // `support_material_1st_layer_flow` (`Flow.cpp:250-260`): the raft
    // flange uses `initial_layer_line_width` when > 0 (else the support
    // line width fallback), not the plain support width.
    let first_layer_width = extrusion_options
        .width_for_role_and_layer(crate::print_paths::PrintPathRole::SupportMaterial, true);
    // `rounded_rectangle_extrusion_spacing` (`Flow.cpp:200-206`) computes
    // `width - height·float(1 − 0.25·π)` in f32 — keep the f32 rounding so
    // `line_half_width`/`length_to_go`/`lerp t` match oracle endpoints.
    let spacing = |width: f64, height: f64| {
        (width as f32 - height as f32 * (1.0f32 - 0.25 * std::f32::consts::PI)) as f64
    };
    let support_flow_spacing = spacing(support_width, layer_height);
    let interface_flow_spacing = spacing(interface_width, layer_height);
    let first_layer_flow_spacing = spacing(first_layer_width, first_height);

    let fill_inputs = RaftFillInputs {
        support_flow_spacing,
        interface_flow_spacing,
        base_pattern_spacing: support_base_pattern_spacing(options),
        interface_spacing: SUPPORT_INTERFACE_SPACING_DEFAULT_MM,
        support_angle_degrees: 0.0,
        base_raft_layers: base_layers,
        interface_raft_layers: interface_layers,
        with_sheath: false,
        honeycomb_base: false,
    };
    let fill_params = fill_inputs.derive();

    let units_per_mm = 1.0 / scale.factor();
    let scaled = |mm: f64| mm * units_per_mm;
    let grid = SupportGridParams::new(
        support_base_pattern_spacing(options) + support_flow_spacing,
        support_flow_spacing,
        units_per_mm,
    );

    let raft_grid = raft_layer_grid(&parameters)?;
    let polygons = raft_polygons(
        &lslices,
        &RaftPolygonParams {
            raft_expansion: scaled(options.raft_expansion_mm()?),
            first_layer_expansion: scaled(options.raft_first_layer_expansion_mm()?),
            raft_layers,
            scale: crate::project_slice::raft::polygons::RaftPolygonScale { units_per_mm },
            grid,
        },
    )
    .map_err(raft_geometry_error)?;

    let plans = raft_layer_plans(
        &lslices,
        &raft_grid,
        &polygons,
        &fill_params,
        support_flow_spacing,
        first_layer_flow_spacing,
        options.raft_first_layer_density_percent()?,
        support_width,
        interface_width,
        extrusion_options
            .width_for_role_and_layer(crate::print_paths::PrintPathRole::SupportMaterial, true),
        first_height,
    );

    Ok(Some(RaftStream {
        plans,
        object_print_z_min: parameters.object_print_z_min,
        support_flow_spacing_mm: support_flow_spacing,
        first_layer_flow_spacing_mm: first_layer_flow_spacing,
    }))
}

pub(super) fn raft_geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("raft geometry error: {error:?}"))
}

pub(super) fn raft_contact_distance(options: &SliceOptions) -> f64 {
    options
        .values()
        .get("raft_contact_distance")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.1)
}

pub(super) fn support_base_pattern_spacing(options: &SliceOptions) -> f64 {
    options
        .values()
        .get("support_base_pattern_spacing")
        .and_then(|value| value.as_f64())
        .unwrap_or(SUPPORT_BASE_PATTERN_SPACING_DEFAULT_MM)
}

mod project;
pub(crate) use project::build_project_raft;
