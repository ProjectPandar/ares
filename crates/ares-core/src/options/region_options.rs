use crate::options::OrcaFloatOrPercentsNullable;

use super::{
    FilamentRegionSourceOptions, FloatOrPercent, OrcaBool, OrcaBoolsNullable, OrcaFloat,
    OrcaFloatsNullable, OrcaInt, OrcaInts, OrcaString, OrcaStrings, Percent,
    ProcessCounterboreHoleBridging, ProcessEnsureVerticalShellThickness, ProcessFuzzySkinMode,
    ProcessFuzzySkinType, ProcessInfillPattern, ProcessIroningType, ProcessNoiseType,
    ProcessRegionSourceOptions, ProcessSeamScarfType, ProcessSurfaceFillOrder,
    ProcessTopSurfaceExpansionDirection, ProcessWallDirection, ProcessWallSequence,
    region_fields::region_option_fields,
};

mod merge;
mod metadata;
pub(crate) use metadata::unescape_strings_cstyle;
mod normalization;
mod overrides;

pub(crate) use overrides::RegionOptionOverrides;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) enum RegionBase<'a> {
    ModelPart {
        process: &'a ProcessRegionSourceOptions,
        object: Option<&'a RegionOptionOverrides>,
        layer_range: Option<&'a RegionOptionOverrides>,
    },
    Modifier {
        parent: &'a RegionOptions,
    },
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) struct RegionOverrideSources<'a> {
    pub(crate) base: RegionBase<'a>,
    pub(crate) volume: &'a RegionOptionOverrides,
    pub(crate) material: Option<&'a RegionOptionOverrides>,
}

macro_rules! declare_region_options {
    ($($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct RegionOptions {
            $(pub $field: $ty),*,
            pub top_surface_fill_order: ProcessSurfaceFillOrder,
            pub bottom_surface_fill_order: ProcessSurfaceFillOrder,
            pub filament_ironing_flow: Percent,
            pub filament_ironing_spacing: OrcaFloat,
            pub filament_ironing_inset: OrcaFloat,
            pub filament_ironing_speed: OrcaFloat,
        }

        impl RegionOptions {
            #[cfg_attr(not(test), allow(dead_code))]
            pub(crate) fn from_base(base: &ProcessRegionSourceOptions) -> Self {
                Self {
                    $($field: base.$field.clone()),*,
                    top_surface_fill_order: base.top_surface_fill_order,
                    bottom_surface_fill_order: base.bottom_surface_fill_order,
                    filament_ironing_flow: base.ironing_flow,
                    filament_ironing_spacing: base.ironing_spacing,
                    filament_ironing_inset: base.ironing_inset,
                    filament_ironing_speed: base.ironing_speed,
                }
            }

            fn from_parent(parent: &Self) -> Self {
                Self {
                    $($field: parent.$field.clone()),*,
                    top_surface_fill_order: parent.top_surface_fill_order,
                    bottom_surface_fill_order: parent.bottom_surface_fill_order,
                    filament_ironing_flow: parent.ironing_flow,
                    filament_ironing_spacing: parent.ironing_spacing,
                    filament_ironing_inset: parent.ironing_inset,
                    filament_ironing_speed: parent.ironing_speed,
                }
            }




        }
    };
}

region_option_fields!(declare_region_options);

impl RegionOptions {
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn resolve(
        filament: &FilamentRegionSourceOptions,
        sources: RegionOverrideSources<'_>,
        num_extruders: usize,
        process_variant_indices: &[usize],
    ) -> Self {
        merge::resolve(filament, sources, num_extruders, process_variant_indices)
    }
}
