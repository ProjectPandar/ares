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

    // `Slicing.cpp:166-172` gap rounding (independent support layer
    // height not configured for the KSR sweep profile).
    let gap_raft_object =
        (raft_contact_distance(options) / layer_height + f64::EPSILON).round() * layer_height;

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
    let support_width =
        extrusion_options.width_for_role(crate::print_paths::PrintPathRole::SupportMaterial);
    let interface_width = extrusion_options
        .width_for_role(crate::print_paths::PrintPathRole::SupportMaterialInterface);
    let first_layer_width = support_width;
    let spacing = |width: f64, height: f64| width - height * (1.0 - std::f64::consts::FRAC_PI_4);
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

    let scaled = |mm: f64| (mm / scale.factor()).round() as i64;
    let grid_resolution = scaled(support_base_pattern_spacing(options) + support_flow_spacing);
    let grid = SupportGridParams::new(grid_resolution, scaled(support_flow_spacing));

    let raft_grid = raft_layer_grid(&parameters)?;
    let polygons = raft_polygons(
        &lslices,
        &RaftPolygonParams {
            raft_expansion: scaled(options.raft_expansion_mm()?),
            first_layer_expansion: scaled(options.raft_first_layer_expansion_mm()?),
            raft_layers,
            scale: crate::project_slice::raft::polygons::RaftPolygonScale {
                units_per_mm: 1.0 / scale.factor(),
            },
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
        first_height,
    );

    Ok(Some(RaftStream {
        plans,
        object_print_z_min: parameters.object_print_z_min,
        support_flow_spacing_mm: support_flow_spacing,
        first_layer_flow_spacing_mm: first_layer_flow_spacing,
    }))
}

fn raft_geometry_error(error: ClipperError) -> SliceError {
    SliceError::InvalidInput(format!("raft geometry error: {error:?}"))
}

fn raft_contact_distance(options: &SliceOptions) -> f64 {
    options
        .values()
        .get("raft_contact_distance")
        .and_then(|value| value.as_f64())
        .unwrap_or(0.1)
}

fn support_base_pattern_spacing(options: &SliceOptions) -> f64 {
    options
        .values()
        .get("support_base_pattern_spacing")
        .and_then(|value| value.as_f64())
        .unwrap_or(SUPPORT_BASE_PATTERN_SPACING_DEFAULT_MM)
}

/// Project-path variant: derives everything from the resolved
/// `ProjectSettings` + `ObjectOptions` (the types the project
/// pipeline carries), with the support-flow width following the
/// header's derivation (`gcode_emit/header.rs:205-221`:
/// `resolved_width(support_line_width, line_width, support_nozzle)`).
pub(crate) fn build_project_raft(
    settings: &crate::ProjectSettings,
    object: &crate::ObjectOptions,
    object_height: f64,
    first_layer_lslices: &[ExPolygon],
    scale: CoordinateScale,
) -> Result<Option<RaftStream>, SliceError> {
    let raft_layers = object.raft_layers.0.max(0) as usize;
    if raft_layers == 0 {
        return Ok(None);
    }
    let parameters =
        crate::project_slice::parameters::slicing_parameters(settings, object, object_height, &[])?;

    let nozzles = &settings.project.print.nozzle_diameter.0;
    let nozzle = nozzles.first().map_or(0.4, |value| value.0);
    let support_nozzle = {
        let index = object.support_filament.0.saturating_sub(1) as usize;
        nozzles
            .get(index)
            .or_else(|| nozzles.first())
            .map_or(nozzle, |value| value.0)
    };
    // `resolved_width(support_line_width, line_width, support_nozzle, 1.0)`
    // (`header.rs:235-244`).
    let resolve_width = |configured: crate::FloatOrPercent| -> f64 {
        explicit_width(configured, support_nozzle)
            .or_else(|| explicit_width(object.line_width, support_nozzle))
            .unwrap_or(support_nozzle)
    };
    let support_width = resolve_width(object.support_line_width);
    let interface_width = support_width; // interface width shares the support width field

    let layer_height = parameters.layer_height;
    let first_height = parameters.first_print_layer_height;
    let spacing = |width: f64, height: f64| width - height * (1.0 - std::f64::consts::FRAC_PI_4);
    let support_flow_spacing = spacing(support_width, layer_height);
    let interface_flow_spacing = spacing(interface_width, layer_height);
    let first_layer_flow_spacing = spacing(support_width, first_height);

    let fill_inputs = RaftFillInputs {
        support_flow_spacing,
        interface_flow_spacing,
        base_pattern_spacing: object.support_base_pattern_spacing.0,
        interface_spacing: object.support_interface_spacing.0,
        support_angle_degrees: object.support_angle.0,
        base_raft_layers: parameters.base_raft_layers,
        interface_raft_layers: parameters.interface_raft_layers,
        with_sheath: false,
        honeycomb_base: false,
    };
    let fill_params = fill_inputs.derive();

    let scaled = |mm: f64| (mm / scale.factor()).round() as i64;
    let grid = SupportGridParams::new(
        scaled(object.support_base_pattern_spacing.0 + support_flow_spacing),
        scaled(support_flow_spacing),
    );
    let raft_grid = raft_layer_grid(&parameters)?;
    let polygons = raft_polygons(
        first_layer_lslices,
        &RaftPolygonParams {
            raft_expansion: scaled(object.raft_expansion.0),
            first_layer_expansion: scaled(object.raft_first_layer_expansion.0),
            raft_layers,
            scale: crate::project_slice::raft::polygons::RaftPolygonScale {
                units_per_mm: 1.0 / scale.factor(),
            },
            grid,
        },
    )
    .map_err(raft_geometry_error)?;
    let plans = raft_layer_plans(
        first_layer_lslices,
        &raft_grid,
        &polygons,
        &fill_params,
        support_flow_spacing,
        first_layer_flow_spacing,
        object.raft_first_layer_density.0,
        support_width,
        interface_width,
        first_height,
    );

    Ok(Some(RaftStream {
        plans,
        object_print_z_min: parameters.object_print_z_min,
        support_flow_spacing_mm: support_flow_spacing,
        first_layer_flow_spacing_mm: first_layer_flow_spacing,
    }))
}

/// `width()` (`header.rs:246-258`): explicit Float/Percent resolution.
fn explicit_width(value: crate::FloatOrPercent, nozzle: f64) -> Option<f64> {
    match value {
        crate::FloatOrPercent::Float(value) if value > 0.0 => Some(value),
        crate::FloatOrPercent::Percent(value) if value.0 > 0.0 => {
            Some(value.0 * f64::from(nozzle as f32) / 100.0)
        }
        _ => None,
    }
}
