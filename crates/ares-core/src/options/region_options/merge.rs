use crate::options::project_variants::apply_nullable_vector;

use super::{
    super::{FilamentRegionSourceOptions, Nullable, OrcaInt, region_fields::region_option_fields},
    RegionBase, RegionOptionOverrides, RegionOptions, RegionOverrideSources, normalization,
};

struct FeatureFilamentOverrideMask {
    sparse_infill_filament_id: bool,
    internal_solid_filament_id: bool,
    top_surface_filament_id: bool,
    bottom_surface_filament_id: bool,
    outer_wall_filament_id: bool,
    inner_wall_filament_id: bool,
}

pub(super) fn resolve(
    filament: &FilamentRegionSourceOptions,
    sources: RegionOverrideSources<'_>,
    num_extruders: usize,
    process_variant_indices: &[usize],
) -> RegionOptions {
    let (mut options, mut mask, layer_range) = match sources.base {
        RegionBase::ModelPart {
            process,
            object,
            layer_range,
        } => {
            let mut options = RegionOptions::from_base(process);
            let mut mask = FeatureFilamentOverrideMask::from_model_part(&options);
            if let Some(object) = object {
                apply_overrides(&mut options, object, &mut mask, process_variant_indices);
            }
            (options, mask, layer_range)
        }
        RegionBase::Modifier { parent } => (
            RegionOptions::from_parent(parent),
            FeatureFilamentOverrideMask::clear(),
            None,
        ),
    };

    apply_overrides(
        &mut options,
        sources.volume,
        &mut mask,
        process_variant_indices,
    );
    if let Some(material) = sources.material {
        apply_overrides(&mut options, material, &mut mask, process_variant_indices);
    }
    if let Some(layer_range) = layer_range {
        apply_overrides(
            &mut options,
            layer_range,
            &mut mask,
            process_variant_indices,
        );
    }
    normalization::normalize(&mut options, num_extruders);
    select_ironing(&mut options, filament);
    options
}

fn select_ironing(options: &mut RegionOptions, filament: &FilamentRegionSourceOptions) {
    // `ConfigOptionVector::get_at` clamps out-of-range indices to the front
    // (`Config.hpp:623-626`): the project may carry fewer vector entries than
    // the extruder an object selects (e.g. a 1-entry nullable from the
    // preset against a second extruder's region).
    let index = options.top_surface_filament_id.0 as usize - 1;
    fn clamp_front<T>(values: &[T], index: usize) -> &T {
        values.get(index).unwrap_or(&values[0])
    }
    options.filament_ironing_flow = match clamp_front(&filament.filament_ironing_flow, index) {
        Nullable::Nil => options.ironing_flow,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_spacing = match clamp_front(&filament.filament_ironing_spacing, index)
    {
        Nullable::Nil => options.ironing_spacing,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_inset = match clamp_front(&filament.filament_ironing_inset, index) {
        Nullable::Nil => options.ironing_inset,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_speed = match clamp_front(&filament.filament_ironing_speed, index) {
        Nullable::Nil => options.ironing_speed,
        Nullable::Value(value) => *value,
    };
}

impl FeatureFilamentOverrideMask {
    fn from_model_part(options: &RegionOptions) -> Self {
        Self {
            sparse_infill_filament_id: options.sparse_infill_filament_id.0 > 0,
            internal_solid_filament_id: options.internal_solid_filament_id.0 > 0,
            top_surface_filament_id: options.top_surface_filament_id.0 > 0,
            bottom_surface_filament_id: options.bottom_surface_filament_id.0 > 0,
            outer_wall_filament_id: options.outer_wall_filament_id.0 > 0,
            inner_wall_filament_id: options.inner_wall_filament_id.0 > 0,
        }
    }

    fn clear() -> Self {
        Self {
            sparse_infill_filament_id: false,
            internal_solid_filament_id: false,
            top_surface_filament_id: false,
            bottom_surface_filament_id: false,
            outer_wall_filament_id: false,
            inner_wall_filament_id: false,
        }
    }
}

fn apply_overrides(
    options: &mut RegionOptions,
    source: &RegionOptionOverrides,
    mask: &mut FeatureFilamentOverrideMask,
    process_variant_indices: &[usize],
) {
    macro_rules! apply_fields {
        ($($field:ident => $key:literal: $ty:ident = $default:expr),* $(,)?) => {
            $(apply_field!(options, source, mask, process_variant_indices, $field, $ty);)*
        };
    }
    region_option_fields!(apply_fields);
    if let Some(order) = source.top_surface_fill_order {
        options.top_surface_fill_order = order;
    }
    if let Some(order) = source.bottom_surface_fill_order {
        options.bottom_surface_fill_order = order;
    }

    if let Some(extruder) = source.extruder.filter(|value| value.0 > 0) {
        apply_fallbacks!(options, mask, extruder;
            sparse_infill_filament_id,
            internal_solid_filament_id,
            top_surface_filament_id,
            bottom_surface_filament_id,
            outer_wall_filament_id,
            inner_wall_filament_id,
        );
    }
}

macro_rules! apply_field {
    ($options:ident, $source:ident, $mask:ident, $indices:ident, sparse_infill_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.sparse_infill_filament_id,
            $source.sparse_infill_filament_id,
            &mut $mask.sparse_infill_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, internal_solid_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.internal_solid_filament_id,
            $source.internal_solid_filament_id,
            &mut $mask.internal_solid_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, top_surface_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.top_surface_filament_id,
            $source.top_surface_filament_id,
            &mut $mask.top_surface_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, bottom_surface_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.bottom_surface_filament_id,
            $source.bottom_surface_filament_id,
            &mut $mask.bottom_surface_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, outer_wall_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.outer_wall_filament_id,
            $source.outer_wall_filament_id,
            &mut $mask.outer_wall_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, inner_wall_filament_id, $ty:ident) => {
        apply_feature(
            &mut $options.inner_wall_filament_id,
            $source.inner_wall_filament_id,
            &mut $mask.inner_wall_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, $field:ident, OrcaFloatsNullable) => {
        if let Some(value) = &$source.$field
            && !$options.$field.source_equals(value)
        {
            apply_nullable_vector(&mut $options.$field.0, &value.0, $indices);
        }
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, $field:ident, OrcaFloatOrPercentsNullable) => {
        if let Some(value) = &$source.$field
            && !$options.$field.source_equals(value)
        {
            apply_nullable_vector(&mut $options.$field.0, &value.0, $indices);
        }
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, $field:ident, OrcaBoolsNullable) => {
        if let Some(value) = &$source.$field
            && $options.$field != *value
        {
            apply_nullable_vector(&mut $options.$field.0, &value.0, $indices);
        }
    };
    ($options:ident, $source:ident, $mask:ident, $indices:ident, $field:ident, $ty:ident) => {
        if let Some(value) = &$source.$field {
            $options.$field = value.clone();
        }
    };
}

macro_rules! apply_fallbacks {
    ($options:ident, $mask:ident, $extruder:ident; $($field:ident),+ $(,)?) => {
        $(if !$mask.$field { $options.$field = $extruder; })+
    };
}

fn apply_feature(output: &mut OrcaInt, override_value: Option<OrcaInt>, is_explicit: &mut bool) {
    if let Some(value) = override_value {
        if value.0 > 0 {
            *output = value;
            *is_explicit = true;
        } else {
            *is_explicit = false;
        }
    }
}

use {apply_fallbacks, apply_field};
