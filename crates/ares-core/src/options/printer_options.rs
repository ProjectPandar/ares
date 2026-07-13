mod gcode_source;
mod machine_envelope;
mod remaining;
mod wire;

pub(crate) use gcode_source::PrinterGCodeSourceOptionsBuilder;
pub use gcode_source::{
    ExtruderType, ExtruderTypes, NozzleType, NullableInts, NullableNozzleTypes,
    PrinterGCodeSourceOptions, PrinterStructure, RetractLiftEnforces, WipeTowerType, ZHopType,
    ZHopTypes,
};
pub(crate) use machine_envelope::MachineEnvelopeOptionsBuilder;
pub use machine_envelope::{InputShaperType, MachineEnvelopeOptions};
pub(crate) use remaining::PrinterRemainingOptionsBuilder;
pub use remaining::{
    AuthorizationType, DefaultBedType, ExtruderVariantLists, NozzleVolumeType, NozzleVolumeTypes,
    NullableFloats, PrintHostType, PrinterModel, PrinterNotes, PrinterRemainingOptions,
    ThumbnailDefinitions,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PrinterOptions {
    pub machine: MachineEnvelopeOptions,
    pub gcode: PrinterGCodeSourceOptions,
    pub remaining: PrinterRemainingOptions,
}

impl Default for PrinterOptions {
    fn default() -> Self {
        PrinterOptionsBuilder::default().resolve()
    }
}

impl<'de> serde::Deserialize<'de> for PrinterOptions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(PrinterVisitor)
    }
}

struct PrinterVisitor;

impl<'de> serde::de::Visitor<'de> for PrinterVisitor {
    type Value = PrinterOptions;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("Orca printer options")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut builder = PrinterOptionsBuilder::default();
        while let Some(key) = map.next_key::<String>()? {
            if !builder.deserialize_known_field(&key, &mut map)? {
                return Err(serde::de::Error::custom(format!(
                    "unknown Orca printer option {key}"
                )));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct PrinterOptionsBuilder {
    machine: MachineEnvelopeOptionsBuilder,
    gcode: PrinterGCodeSourceOptionsBuilder,
    remaining: PrinterRemainingOptionsBuilder,
}

impl PrinterOptionsBuilder {
    pub(crate) fn deserialize_known_field<'de, A>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        if self.machine.deserialize_known_field(key, map)?
            || self.gcode.deserialize_known_field(key, map)?
        {
            Ok(true)
        } else {
            self.remaining.deserialize_known_field(key, map)
        }
    }

    pub(crate) fn resolve(self) -> PrinterOptions {
        PrinterOptions {
            machine: self.machine.resolve(),
            gcode: self.gcode.resolve(),
            remaining: self.remaining.resolve(),
        }
    }
}
