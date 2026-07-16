pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use crate::SliceError;

use super::super::{
    Nullable, OrcaFloat, Percent,
    option_group::{
        apply_variant_slots, declare_option_group, normalize_present_variant_vector,
        normalize_root_variant_vector, nullable_float_variant_vectors_equal,
        nullable_percent_variant_vectors_equal,
    },
};

declare_option_group! {
    append pub struct FilamentRegionSourceOptions, FilamentRegionSourceOptionsBuilder {
        filament_ironing_flow => "filament_ironing_flow": Vec<Nullable<Percent>> = nil_vector(),
        filament_ironing_spacing => "filament_ironing_spacing": Vec<Nullable<OrcaFloat>> = nil_vector(),
        filament_ironing_inset => "filament_ironing_inset": Vec<Nullable<OrcaFloat>> = nil_vector(),
        filament_ironing_speed => "filament_ironing_speed": Vec<Nullable<OrcaFloat>> = nil_vector(),
    }
}

macro_rules! region_profile_fields {
    ($callback:ident $(, $argument:expr)*) => {
        $callback! {
            [$($argument),*]
            {
                filament_ironing_flow => ("filament_ironing_flow", nullable_percent_variant_vectors_equal),
                filament_ironing_spacing => ("filament_ironing_spacing", nullable_float_variant_vectors_equal),
                filament_ironing_inset => ("filament_ironing_inset", nullable_float_variant_vectors_equal),
                filament_ironing_speed => ("filament_ironing_speed", nullable_float_variant_vectors_equal),
            }
        }
    };
}

macro_rules! normalize_region_root_field {
    ([$target:expr, $defaults:expr, $count:expr] {$($field:ident => ($key:literal, $equal:path)),* $(,)?}) => {
        $(
            normalize_root_variant_vector(
                &mut $target.$field,
                &$defaults.$field,
                $count,
                $key,
                |values| values.iter().all(|value| matches!(value, Nullable::Nil)),
            )?;
        )*
    };
}

macro_rules! normalize_region_child_field {
    ([$builder:expr, $count:expr] {$($field:ident => ($key:literal, $equal:path)),* $(,)?}) => {
        $(
            if let Some(values) = $builder.$field.as_mut() {
                normalize_present_variant_vector(values, $count, $key)?;
            }
        )*
    };
}

macro_rules! apply_region_profile_field {
    ([$builder:expr, $target:expr, $mapping:expr] {$($field:ident => ($key:literal, $equal:path)),* $(,)?}) => {
        $(let $field = $builder.$field.take();)*
        $builder.apply_present($target);
        $(
            if let Some(child) = $field {
                apply_variant_slots(
                    &mut $target.$field,
                    &child,
                    $mapping,
                    $key,
                    ($equal, |value| matches!(value, Nullable::Value(_))),
                )?;
            }
        )*
    };
}

impl FilamentRegionSourceOptions {
    pub(super) fn normalize_profile_root(
        &mut self,
        defaults: &Self,
        count: usize,
    ) -> Result<(), SliceError> {
        region_profile_fields!(normalize_region_root_field, self, defaults, count);
        Ok(())
    }
}

impl FilamentRegionSourceOptionsBuilder {
    pub(super) fn normalize_profile_child(&mut self, count: usize) -> Result<(), SliceError> {
        region_profile_fields!(normalize_region_child_field, self, count);
        Ok(())
    }

    pub(super) fn apply_profile_child(
        mut self,
        target: &mut FilamentRegionSourceOptions,
        mapping: &[Option<usize>],
    ) -> Result<(), SliceError> {
        region_profile_fields!(apply_region_profile_field, self, target, mapping);
        Ok(())
    }
}

impl FilamentRegionSourceOptions {
    pub const DECLARATION_ORDER: [&'static str; 4] = [
        "filament_ironing_flow",
        "filament_ironing_spacing",
        "filament_ironing_inset",
        "filament_ironing_speed",
    ];
}

impl Default for FilamentRegionSourceOptions {
    fn default() -> Self {
        FilamentRegionSourceOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentRegionSourceOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RegionSourceVisitor)
    }
}

struct RegionSourceVisitor;

impl<'de> Visitor<'de> for RegionSourceVisitor {
    type Value = FilamentRegionSourceOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca PrintRegionConfig filament options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentRegionSourceOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &Self::Value::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn nil_vector<T>() -> Vec<Nullable<T>> {
    vec![Nullable::Nil]
}
