use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{MapAccess, Visitor},
    ser::SerializeMap,
};

use super::super::{
    config_types::{OrcaBool, OrcaFloat, OrcaFloats},
    option_group::declare_option_group,
};

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum InputShaperType {
    #[default]
    Default,
    #[serde(rename = "MZV")]
    Mzv,
    #[serde(rename = "ZV")]
    Zv,
    #[serde(rename = "ZVD")]
    Zvd,
    #[serde(rename = "ZVDD")]
    Zvdd,
    #[serde(rename = "ZVDDD")]
    Zvddd,
    #[serde(rename = "EI")]
    Ei,
    #[serde(rename = "EI2")]
    Ei2,
    #[serde(rename = "2HUMP_EI")]
    TwoHumpEi,
    #[serde(rename = "EI3")]
    Ei3,
    #[serde(rename = "3HUMP_EI")]
    ThreeHumpEi,
    #[serde(rename = "DAA")]
    Daa,
    Disable,
}

declare_option_group! {
    pub struct MachineEnvelopeOptions, MachineEnvelopeOptionsBuilder {
        emit_machine_limits_to_gcode => "emit_machine_limits_to_gcode": OrcaBool = OrcaBool(true),
        machine_max_acceleration_x => "machine_max_acceleration_x": OrcaFloats = floats(&[1000.0, 1000.0]),
        machine_max_acceleration_y => "machine_max_acceleration_y": OrcaFloats = floats(&[1000.0, 1000.0]),
        machine_max_acceleration_z => "machine_max_acceleration_z": OrcaFloats = floats(&[500.0, 200.0]),
        machine_max_acceleration_e => "machine_max_acceleration_e": OrcaFloats = floats(&[5000.0, 5000.0]),
        machine_max_speed_x => "machine_max_speed_x": OrcaFloats = floats(&[500.0, 200.0]),
        machine_max_speed_y => "machine_max_speed_y": OrcaFloats = floats(&[500.0, 200.0]),
        machine_max_speed_z => "machine_max_speed_z": OrcaFloats = floats(&[12.0, 12.0]),
        machine_max_speed_e => "machine_max_speed_e": OrcaFloats = floats(&[120.0, 120.0]),
        machine_max_acceleration_extruding => "machine_max_acceleration_extruding": OrcaFloats = floats(&[1500.0, 1250.0]),
        machine_max_acceleration_retracting => "machine_max_acceleration_retracting": OrcaFloats = floats(&[1500.0, 1250.0]),
        machine_max_acceleration_travel => "machine_max_acceleration_travel": OrcaFloats = floats(&[0.0, 0.0]),
        machine_max_jerk_x => "machine_max_jerk_x": OrcaFloats = floats(&[10.0, 10.0]),
        machine_max_jerk_y => "machine_max_jerk_y": OrcaFloats = floats(&[10.0, 10.0]),
        machine_max_jerk_z => "machine_max_jerk_z": OrcaFloats = floats(&[0.2, 0.4]),
        machine_max_jerk_e => "machine_max_jerk_e": OrcaFloats = floats(&[2.5, 2.5]),
        machine_max_junction_deviation => "machine_max_junction_deviation": OrcaFloats = floats(&[0.01]),
        machine_min_travel_rate => "machine_min_travel_rate": OrcaFloats = floats(&[0.0, 0.0]),
        machine_min_extruding_rate => "machine_min_extruding_rate": OrcaFloats = floats(&[0.0, 0.0]),
        resonance_avoidance => "resonance_avoidance": OrcaBool = OrcaBool(false),
        min_resonance_avoidance_speed => "min_resonance_avoidance_speed": OrcaFloat = OrcaFloat(70.0),
        max_resonance_avoidance_speed => "max_resonance_avoidance_speed": OrcaFloat = OrcaFloat(120.0),
        input_shaping_emit => "input_shaping_emit": OrcaBool = OrcaBool(false),
        input_shaping_type => "input_shaping_type": InputShaperType = InputShaperType::Default,
        input_shaping_freq_x => "input_shaping_freq_x": OrcaFloat = OrcaFloat(0.0),
        input_shaping_freq_y => "input_shaping_freq_y": OrcaFloat = OrcaFloat(0.0),
        input_shaping_damp_x => "input_shaping_damp_x": OrcaFloat = OrcaFloat(0.1),
        input_shaping_damp_y => "input_shaping_damp_y": OrcaFloat = OrcaFloat(0.1),
    }
}

impl MachineEnvelopeOptions {
    pub const DECLARATION_ORDER: [&'static str; 28] = [
        "emit_machine_limits_to_gcode",
        "machine_max_acceleration_x",
        "machine_max_acceleration_y",
        "machine_max_acceleration_z",
        "machine_max_acceleration_e",
        "machine_max_speed_x",
        "machine_max_speed_y",
        "machine_max_speed_z",
        "machine_max_speed_e",
        "machine_max_acceleration_extruding",
        "machine_max_acceleration_retracting",
        "machine_max_acceleration_travel",
        "machine_max_jerk_x",
        "machine_max_jerk_y",
        "machine_max_jerk_z",
        "machine_max_jerk_e",
        "machine_max_junction_deviation",
        "machine_min_travel_rate",
        "machine_min_extruding_rate",
        "resonance_avoidance",
        "min_resonance_avoidance_speed",
        "max_resonance_avoidance_speed",
        "input_shaping_emit",
        "input_shaping_type",
        "input_shaping_freq_x",
        "input_shaping_freq_y",
        "input_shaping_damp_x",
        "input_shaping_damp_y",
    ];
}

impl Default for MachineEnvelopeOptions {
    fn default() -> Self {
        MachineEnvelopeOptionsBuilder::default().resolve()
    }
}

impl Serialize for MachineEnvelopeOptions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(28))?;
        map.serialize_entry(
            "emit_machine_limits_to_gcode",
            &self.emit_machine_limits_to_gcode,
        )?;
        map.serialize_entry("input_shaping_damp_x", &self.input_shaping_damp_x)?;
        map.serialize_entry("input_shaping_damp_y", &self.input_shaping_damp_y)?;
        map.serialize_entry("input_shaping_emit", &self.input_shaping_emit)?;
        map.serialize_entry("input_shaping_freq_x", &self.input_shaping_freq_x)?;
        map.serialize_entry("input_shaping_freq_y", &self.input_shaping_freq_y)?;
        map.serialize_entry("input_shaping_type", &self.input_shaping_type)?;
        map.serialize_entry(
            "machine_max_acceleration_e",
            &self.machine_max_acceleration_e,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_extruding",
            &self.machine_max_acceleration_extruding,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_retracting",
            &self.machine_max_acceleration_retracting,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_travel",
            &self.machine_max_acceleration_travel,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_x",
            &self.machine_max_acceleration_x,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_y",
            &self.machine_max_acceleration_y,
        )?;
        map.serialize_entry(
            "machine_max_acceleration_z",
            &self.machine_max_acceleration_z,
        )?;
        map.serialize_entry("machine_max_jerk_e", &self.machine_max_jerk_e)?;
        map.serialize_entry("machine_max_jerk_x", &self.machine_max_jerk_x)?;
        map.serialize_entry("machine_max_jerk_y", &self.machine_max_jerk_y)?;
        map.serialize_entry("machine_max_jerk_z", &self.machine_max_jerk_z)?;
        map.serialize_entry(
            "machine_max_junction_deviation",
            &self.machine_max_junction_deviation,
        )?;
        map.serialize_entry("machine_max_speed_e", &self.machine_max_speed_e)?;
        map.serialize_entry("machine_max_speed_x", &self.machine_max_speed_x)?;
        map.serialize_entry("machine_max_speed_y", &self.machine_max_speed_y)?;
        map.serialize_entry("machine_max_speed_z", &self.machine_max_speed_z)?;
        map.serialize_entry(
            "machine_min_extruding_rate",
            &self.machine_min_extruding_rate,
        )?;
        map.serialize_entry("machine_min_travel_rate", &self.machine_min_travel_rate)?;
        map.serialize_entry(
            "max_resonance_avoidance_speed",
            &self.max_resonance_avoidance_speed,
        )?;
        map.serialize_entry(
            "min_resonance_avoidance_speed",
            &self.min_resonance_avoidance_speed,
        )?;
        map.serialize_entry("resonance_avoidance", &self.resonance_avoidance)?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for MachineEnvelopeOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MachineEnvelopeVisitor)
    }
}

struct MachineEnvelopeVisitor;

impl<'de> Visitor<'de> for MachineEnvelopeVisitor {
    type Value = MachineEnvelopeOptions;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Orca MachineEnvelopeConfig options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut builder = MachineEnvelopeOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &MachineEnvelopeOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

fn floats(values: &[f64]) -> OrcaFloats {
    OrcaFloats(values.iter().copied().map(OrcaFloat).collect())
}
