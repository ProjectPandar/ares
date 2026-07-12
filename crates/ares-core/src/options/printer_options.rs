mod machine_envelope;

pub(crate) use machine_envelope::MachineEnvelopeOptionsBuilder;
pub use machine_envelope::{InputShaperType, MachineEnvelopeOptions};

#[derive(Clone, Debug, PartialEq)]
pub struct PrinterOptions {
    pub machine: MachineEnvelopeOptions,
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
                return Err(serde::de::Error::unknown_field(
                    &key,
                    &MachineEnvelopeOptions::DECLARATION_ORDER,
                ));
            }
        }
        Ok(builder.resolve())
    }
}

#[derive(Default)]
pub(crate) struct PrinterOptionsBuilder {
    machine: MachineEnvelopeOptionsBuilder,
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
        self.machine.deserialize_known_field(key, map)
    }

    pub(crate) fn resolve(self) -> PrinterOptions {
        PrinterOptions {
            machine: self.machine.resolve(),
        }
    }
}
