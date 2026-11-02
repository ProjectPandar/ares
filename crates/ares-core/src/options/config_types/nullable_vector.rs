use serde::{
    Deserialize, Deserializer,
    de::{DeserializeOwned, Error, SeqAccess, Visitor, value::SeqAccessDeserializer},
};

use super::{Nullable, OrcaBool, OrcaFloat};
use crate::options::{json_deserialize, registry::OptionValueKind};

pub(super) fn bools<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Nullable<OrcaBool>>, D::Error> {
    deserialize(deserializer, OptionValueKind::BoolsNullable)
}

pub(super) fn floats<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<Nullable<OrcaFloat>>, D::Error> {
    deserialize(deserializer, OptionValueKind::FloatsNullable)
}

// Config.cpp:981-983,626-695: a JSON string reaches the vector's own
// deserialize, just as the joined wire of a JSON array does.
fn deserialize<'de, D: Deserializer<'de>, T: DeserializeOwned>(
    deserializer: D,
    kind: OptionValueKind,
) -> Result<Vec<T>, D::Error> {
    struct VectorVisitor<T> {
        kind: OptionValueKind,
        item: std::marker::PhantomData<T>,
    }

    impl<'de, T: DeserializeOwned> Visitor<'de> for VectorVisitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("a nullable vector sequence or comma-separated wire string")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, sequence: A) -> Result<Self::Value, A::Error> {
            Vec::deserialize(SeqAccessDeserializer::new(sequence))
        }

        fn visit_str<E: Error>(self, wire: &str) -> Result<Self::Value, E> {
            let decoded = json_deserialize::decode(self.kind, wire).map_err(E::custom)?;
            Vec::deserialize(decoded).map_err(E::custom)
        }

        fn visit_string<E: Error>(self, wire: String) -> Result<Self::Value, E> {
            self.visit_str(&wire)
        }
    }

    deserializer.deserialize_any(VectorVisitor {
        kind,
        item: std::marker::PhantomData,
    })
}
