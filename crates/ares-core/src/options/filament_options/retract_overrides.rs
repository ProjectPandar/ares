pub(crate) mod wire;

use std::fmt;

use serde::{Deserialize, Deserializer, de::Visitor};

use super::super::{
    Nullable, OrcaBool, OrcaFloat, Percent, RetractLiftEnforce, ZHopType,
    option_group::declare_option_group,
};

declare_option_group! {
    pub struct FilamentRetractOverrideOptions, FilamentRetractOverrideOptionsBuilder {
        filament_retraction_length => "filament_retraction_length": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.8]),
        filament_z_hop => "filament_z_hop": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.4]),
        filament_z_hop_types => "filament_z_hop_types": Vec<Nullable<ZHopType>> = nullable_values(&[ZHopType::Slope]),
        filament_retract_lift_above => "filament_retract_lift_above": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.0]),
        filament_retract_lift_below => "filament_retract_lift_below": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.0]),
        filament_retract_lift_enforce => "filament_retract_lift_enforce": Vec<Nullable<RetractLiftEnforce>> = nullable_values(&[RetractLiftEnforce::AllSurfaces]),
        filament_retraction_speed => "filament_retraction_speed": Vec<Nullable<OrcaFloat>> = nullable_floats(&[30.0]),
        filament_deretraction_speed => "filament_deretraction_speed": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.0]),
        filament_retract_restart_extra => "filament_retract_restart_extra": Vec<Nullable<OrcaFloat>> = nullable_floats(&[0.0]),
        filament_retraction_minimum_travel => "filament_retraction_minimum_travel": Vec<Nullable<OrcaFloat>> = nullable_floats(&[2.0]),
        filament_wipe_distance => "filament_wipe_distance": Vec<Nullable<OrcaFloat>> = nullable_floats(&[1.0]),
        filament_retract_when_changing_layer => "filament_retract_when_changing_layer": Vec<Nullable<OrcaBool>> = nullable_bools(&[false]),
        filament_wipe => "filament_wipe": Vec<Nullable<OrcaBool>> = nullable_bools(&[false]),
        filament_retract_before_wipe => "filament_retract_before_wipe": Vec<Nullable<Percent>> = nullable_percents(&[100.0]),
        filament_long_retractions_when_cut => "filament_long_retractions_when_cut": Vec<Nullable<OrcaBool>> = nullable_bools(&[false]),
        filament_retraction_distances_when_cut => "filament_retraction_distances_when_cut": Vec<Nullable<OrcaFloat>> = nullable_floats(&[18.0]),
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

fn nullable_bools(values: &[bool]) -> Vec<Nullable<OrcaBool>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(OrcaBool(value)))
        .collect()
}

fn nullable_floats(values: &[f64]) -> Vec<Nullable<OrcaFloat>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(OrcaFloat(value)))
        .collect()
}

fn nullable_percents(values: &[f64]) -> Vec<Nullable<Percent>> {
    values
        .iter()
        .copied()
        .map(|value| Nullable::Value(Percent(value)))
        .collect()
}

fn nullable_values<T: Copy>(values: &[T]) -> Vec<Nullable<T>> {
    values.iter().copied().map(Nullable::Value).collect()
}
