use crate::{
    ObjectOptions, SliceError, geometry::CoordinateScale,
    project::effective_config::types::ResolvedProjectObject,
};

use super::{
    layers::PlannedLayer,
    region_slices::{PostRegion, PostRegionPrintObject},
};

pub(super) mod geometry;

use geometry::{
    ConicalOverhangGeometry, apply_conical_overhang_layer_pair, conical_geometry_error,
    derive_conical_overhang_geometry,
};

const INVALID_ANGLE: &str = "invalid Orca option make_overhang_printable_angle";
const INVALID_HOLE_SIZE: &str = "invalid Orca option make_overhang_printable_hole_size";

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct ValidatedConicalOverhangOptions {
    pub(super) angle_degrees: f64,
    pub(super) hole_size_mm2: f64,
    pub(super) layer_height_mm: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum ConicalOverhangStage {
    Empty,
    AngleNinety,
    Geometry(ConicalOverhangGeometry),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LayerPairClassification {
    UpperEmpty,
    CurrentGated,
    Geometry,
}

pub(super) fn validate_conical_overhang_options(
    objects: &[&ObjectOptions],
) -> Result<Vec<ValidatedConicalOverhangOptions>, SliceError> {
    objects
        .iter()
        .map(|options| {
            let angle_degrees = options.make_overhang_printable_angle.0;
            if !angle_degrees.is_finite() || !(0.0..=90.0).contains(&angle_degrees) {
                return Err(invalid(INVALID_ANGLE));
            }

            let hole_size_mm2 = options.make_overhang_printable_hole_size.0;
            if !hole_size_mm2.is_finite() || hole_size_mm2 < 0.0 {
                return Err(invalid(INVALID_HOLE_SIZE));
            }

            Ok(ValidatedConicalOverhangOptions {
                angle_degrees,
                hole_size_mm2,
                layer_height_mm: options.layer_height.0,
            })
        })
        .collect()
}

pub(super) fn classify_conical_overhang_stage(
    options: ValidatedConicalOverhangOptions,
    retained_layers: &[PlannedLayer],
    scale: CoordinateScale,
) -> Result<ConicalOverhangStage, SliceError> {
    if retained_layers.is_empty() {
        return Ok(ConicalOverhangStage::Empty);
    }
    if options.angle_degrees == 90.0 {
        return Ok(ConicalOverhangStage::AngleNinety);
    }
    derive_conical_overhang_geometry(options, scale).map(ConicalOverhangStage::Geometry)
}

pub(super) fn classify_layer_pair(
    regions: &[PostRegion],
    current_layer_index: usize,
    upper_layer_index: usize,
) -> LayerPairClassification {
    if regions
        .iter()
        .all(|region| region.layers[upper_layer_index].surfaces.is_empty())
    {
        return LayerPairClassification::UpperEmpty;
    }
    if regions.iter().all(|region| {
        region.layers[current_layer_index].surfaces.is_empty()
            || !region.options.make_overhang_printable.0
    }) {
        return LayerPairClassification::CurrentGated;
    }
    LayerPairClassification::Geometry
}

pub(super) fn apply_project_conical_overhang(
    objects: &mut [PostRegionPrintObject],
    resolved_objects: &[ResolvedProjectObject],
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let validated = validate_conical_overhang_options(
        &resolved_objects
            .iter()
            .map(|resolved| &resolved.object)
            .collect::<Vec<_>>(),
    )?;
    let flattened = resolved_objects
        .iter()
        .zip(validated)
        .flat_map(|(resolved, options)| std::iter::repeat_n(options, resolved.print_objects.len()))
        .collect::<Vec<_>>();
    assert_eq!(
        objects.len(),
        flattened.len(),
        "post-region object count must match resolved print instances"
    );

    for (object, options) in objects.iter_mut().zip(flattened) {
        apply_conical_overhang(object, options, scale)?;
    }
    Ok(())
}

fn apply_conical_overhang(
    object: &mut PostRegionPrintObject,
    options: ValidatedConicalOverhangOptions,
    scale: CoordinateScale,
) -> Result<(), SliceError> {
    let geometry = match classify_conical_overhang_stage(options, &object.plan.layers, scale)? {
        ConicalOverhangStage::Empty | ConicalOverhangStage::AngleNinety => return Ok(()),
        ConicalOverhangStage::Geometry(geometry) => geometry,
    };

    for upper_layer_index in (1..object.plan.layers.len()).rev() {
        let current_layer_index = upper_layer_index - 1;
        if classify_layer_pair(&object.regions, current_layer_index, upper_layer_index)
            != LayerPairClassification::Geometry
        {
            continue;
        }
        apply_conical_overhang_layer_pair(
            &mut object.regions,
            current_layer_index,
            upper_layer_index,
            geometry,
        )
        .map_err(|_| conical_geometry_error())?;
    }
    Ok(())
}

fn invalid(message: &str) -> SliceError {
    SliceError::InvalidInput(message.to_owned())
}

const _: fn(&[&ObjectOptions]) -> Result<Vec<ValidatedConicalOverhangOptions>, SliceError> =
    validate_conical_overhang_options;
const _: fn(
    ValidatedConicalOverhangOptions,
    &[PlannedLayer],
    CoordinateScale,
) -> Result<ConicalOverhangStage, SliceError> = classify_conical_overhang_stage;
const _: fn(&[PostRegion], usize, usize) -> LayerPairClassification = classify_layer_pair;
type ApplyProjectConicalOverhang = fn(
    &mut [PostRegionPrintObject],
    &[ResolvedProjectObject],
    CoordinateScale,
) -> Result<(), SliceError>;
const _: ApplyProjectConicalOverhang = apply_project_conical_overhang;
