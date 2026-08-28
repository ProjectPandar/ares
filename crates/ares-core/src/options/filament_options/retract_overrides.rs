pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use crate::SliceError;

use super::super::{
    Nullable, OrcaBool, OrcaFloat, Percent, RetractLiftEnforce, ZHopType,
    option_group::{
        apply_variant_slots, declare_option_group, exact_variant_vectors_equal,
        normalize_present_variant_vector, normalize_root_variant_vector,
        nullable_float_variant_vectors_equal, nullable_percent_variant_vectors_equal,
    },
};

declare_option_group! {
    append pub struct FilamentRetractOverrideOptions, FilamentRetractOverrideOptionsBuilder {
        filament_retraction_length => "filament_retraction_length": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_z_hop => "filament_z_hop": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_z_hop_types => "filament_z_hop_types": Vec<Nullable<ZHopType>> = nullable_nil(),
        filament_retract_lift_above => "filament_retract_lift_above": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_retract_lift_below => "filament_retract_lift_below": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_retract_lift_enforce => "filament_retract_lift_enforce": Vec<Nullable<RetractLiftEnforce>> = nullable_nil(),
        filament_retraction_speed => "filament_retraction_speed": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_deretraction_speed => "filament_deretraction_speed": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_retract_restart_extra => "filament_retract_restart_extra": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_retraction_minimum_travel => "filament_retraction_minimum_travel": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_wipe_distance => "filament_wipe_distance": Vec<Nullable<OrcaFloat>> = nullable_nil(),
        filament_retract_when_changing_layer => "filament_retract_when_changing_layer": Vec<Nullable<OrcaBool>> = nullable_nil(),
        filament_wipe => "filament_wipe": Vec<Nullable<OrcaBool>> = nullable_nil(),
        filament_retract_before_wipe => "filament_retract_before_wipe": Vec<Nullable<Percent>> = nullable_nil(),
        filament_long_retractions_when_cut => "filament_long_retractions_when_cut": Vec<Nullable<OrcaBool>> = nullable_nil(),
        filament_retraction_distances_when_cut => "filament_retraction_distances_when_cut": Vec<Nullable<OrcaFloat>> = nullable_nil(),
    }
}

macro_rules! retract_profile_fields {
    ($callback:ident $(, $argument:expr)*) => {
        $callback! {
            [$($argument),*]
            {
                filament_retraction_length => ("filament_retraction_length", nullable_float_variant_vectors_equal),
                filament_z_hop => ("filament_z_hop", nullable_float_variant_vectors_equal),
                filament_z_hop_types => ("filament_z_hop_types", exact_variant_vectors_equal),
                filament_retract_lift_above => ("filament_retract_lift_above", nullable_float_variant_vectors_equal),
                filament_retract_lift_below => ("filament_retract_lift_below", nullable_float_variant_vectors_equal),
                filament_retract_lift_enforce => ("filament_retract_lift_enforce", exact_variant_vectors_equal),
                filament_retraction_speed => ("filament_retraction_speed", nullable_float_variant_vectors_equal),
                filament_deretraction_speed => ("filament_deretraction_speed", nullable_float_variant_vectors_equal),
                filament_retract_restart_extra => ("filament_retract_restart_extra", nullable_float_variant_vectors_equal),
                filament_retraction_minimum_travel => ("filament_retraction_minimum_travel", nullable_float_variant_vectors_equal),
                filament_wipe_distance => ("filament_wipe_distance", nullable_float_variant_vectors_equal),
                filament_retract_when_changing_layer => ("filament_retract_when_changing_layer", exact_variant_vectors_equal),
                filament_wipe => ("filament_wipe", exact_variant_vectors_equal),
                filament_retract_before_wipe => ("filament_retract_before_wipe", nullable_percent_variant_vectors_equal),
                filament_long_retractions_when_cut => ("filament_long_retractions_when_cut", exact_variant_vectors_equal),
                filament_retraction_distances_when_cut => ("filament_retraction_distances_when_cut", nullable_float_variant_vectors_equal),
            }
        }
    };
}

macro_rules! normalize_retract_root_field {
    ([$target:expr, $defaults:expr, $count:expr] {$($field:ident => ($key:literal, $equal:path)),* $(,)?}) => {
        $(
            normalize_root_variant_vector(
                &mut $target.$field,
                &$defaults.$field,
                $count,
                $key,
                |_| false,
            )?;
        )*
    };
}

macro_rules! normalize_retract_child_field {
    ([$builder:expr, $count:expr] {$($field:ident => ($key:literal, $equal:path)),* $(,)?}) => {
        $(
            if let Some(values) = $builder.$field.as_mut() {
                normalize_present_variant_vector(values, $count, $key)?;
            }
        )*
    };
}

macro_rules! apply_retract_profile_field {
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

impl FilamentRetractOverrideOptions {
    pub(super) fn normalize_profile_root(
        &mut self,
        defaults: &Self,
        count: usize,
    ) -> Result<(), SliceError> {
        retract_profile_fields!(normalize_retract_root_field, self, defaults, count);
        Ok(())
    }
}

impl FilamentRetractOverrideOptionsBuilder {
    pub(super) fn normalize_profile_child(&mut self, count: usize) -> Result<(), SliceError> {
        retract_profile_fields!(normalize_retract_child_field, self, count);
        Ok(())
    }

    pub(super) fn apply_profile_child(
        mut self,
        target: &mut FilamentRetractOverrideOptions,
        mapping: &[Option<usize>],
    ) -> Result<(), SliceError> {
        retract_profile_fields!(apply_retract_profile_field, self, target, mapping);
        Ok(())
    }
}

impl FilamentRetractOverrideOptions {
    pub const DECLARATION_ORDER: [&'static str; 16] = [
        "filament_retraction_length",
        "filament_z_hop",
        "filament_z_hop_types",
        "filament_retract_lift_above",
        "filament_retract_lift_below",
        "filament_retract_lift_enforce",
        "filament_retraction_speed",
        "filament_deretraction_speed",
        "filament_retract_restart_extra",
        "filament_retraction_minimum_travel",
        "filament_wipe_distance",
        "filament_retract_when_changing_layer",
        "filament_wipe",
        "filament_retract_before_wipe",
        "filament_long_retractions_when_cut",
        "filament_retraction_distances_when_cut",
    ];
}

impl Default for FilamentRetractOverrideOptions {
    fn default() -> Self {
        FilamentRetractOverrideOptionsBuilder::default().resolve()
    }
}

impl<'de> Deserialize<'de> for FilamentRetractOverrideOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RetractOverrideVisitor)
    }
}

struct RetractOverrideVisitor;

impl<'de> Visitor<'de> for RetractOverrideVisitor {
    type Value = FilamentRetractOverrideOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca generated filament retract overrides")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = FilamentRetractOverrideOptionsBuilder::default();
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

fn nullable_nil<T>() -> Vec<Nullable<T>> {
    vec![Nullable::Nil]
}
