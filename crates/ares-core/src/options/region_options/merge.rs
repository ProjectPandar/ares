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
                apply_overrides(&mut options, object, &mut mask);
            }
            (options, mask, layer_range)
        }
        RegionBase::Modifier { parent } => (
            RegionOptions::from_parent(parent),
            FeatureFilamentOverrideMask::clear(),
            None,
        ),
    };

    apply_overrides(&mut options, sources.volume, &mut mask);
    if let Some(material) = sources.material {
        apply_overrides(&mut options, material, &mut mask);
    }
    if let Some(layer_range) = layer_range {
        apply_overrides(&mut options, layer_range, &mut mask);
    }
    normalization::normalize(&mut options, num_extruders);
    select_ironing(&mut options, filament);
    options
}

fn select_ironing(options: &mut RegionOptions, filament: &FilamentRegionSourceOptions) {
    let index = options.top_surface_filament_id.0 as usize - 1;
    options.filament_ironing_flow = match &filament.filament_ironing_flow[index] {
        Nullable::Nil => options.ironing_flow,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_spacing = match &filament.filament_ironing_spacing[index] {
        Nullable::Nil => options.ironing_spacing,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_inset = match &filament.filament_ironing_inset[index] {
        Nullable::Nil => options.ironing_inset,
        Nullable::Value(value) => *value,
    };
    options.filament_ironing_speed = match &filament.filament_ironing_speed[index] {
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
) {
    macro_rules! apply_fields {
        ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
            $(apply_field!(options, source, mask, $field);)*
        };
    }
    region_option_fields!(apply_fields);

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
    ($options:ident, $source:ident, $mask:ident, sparse_infill_filament_id) => {
        apply_feature(
            &mut $options.sparse_infill_filament_id,
            $source.sparse_infill_filament_id,
            &mut $mask.sparse_infill_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, internal_solid_filament_id) => {
        apply_feature(
            &mut $options.internal_solid_filament_id,
            $source.internal_solid_filament_id,
            &mut $mask.internal_solid_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, top_surface_filament_id) => {
        apply_feature(
            &mut $options.top_surface_filament_id,
            $source.top_surface_filament_id,
            &mut $mask.top_surface_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, bottom_surface_filament_id) => {
        apply_feature(
            &mut $options.bottom_surface_filament_id,
            $source.bottom_surface_filament_id,
            &mut $mask.bottom_surface_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, outer_wall_filament_id) => {
        apply_feature(
            &mut $options.outer_wall_filament_id,
            $source.outer_wall_filament_id,
            &mut $mask.outer_wall_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, inner_wall_filament_id) => {
        apply_feature(
            &mut $options.inner_wall_filament_id,
            $source.inner_wall_filament_id,
            &mut $mask.inner_wall_filament_id,
        );
    };
    ($options:ident, $source:ident, $mask:ident, $field:ident) => {
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
