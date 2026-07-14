mod lexical;

use serde::de::{
    DeserializeSeed, Error, IgnoredAny, MapAccess, SeqAccess, Visitor, value::SeqAccessDeserializer,
};

use super::{
    EXPLICIT_RULES, LegacyAction, LegacyOutcome, LegacyRule, LegacyTransformError,
    transform_json_array, transform_lexical, transform_obsolete,
};
use crate::options::project_settings::ProjectSettingsBuilder;

pub(crate) fn deserialize_project_field<'de, A>(
    builder: &mut ProjectSettingsBuilder,
    key: &str,
    map: &mut A,
) -> Result<bool, A::Error>
where
    A: MapAccess<'de>,
{
    if let Some(rule) = EXPLICIT_RULES.iter().find(|rule| rule.source == key) {
        if matches!(rule.action, LegacyAction::DeferredProfileBookkeeping { .. }) {
            map.next_value::<IgnoredAny>()?;
            return Err(A::Error::custom(format_args!(
                "unsupported deferred Orca project option {}",
                rule.source
            )));
        }
        map.next_value_seed(LegacyValueSeed { builder, rule })?;
        return Ok(true);
    }

    if transform_obsolete(key).is_some() {
        map.next_value_seed(ObsoleteValueSeed { source: key })?;
        return Ok(true);
    }

    Ok(false)
}

struct ObsoleteValueSeed<'a> {
    source: &'a str,
}

impl<'de> DeserializeSeed<'de> for ObsoleteValueSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(ObsoleteValueVisitor {
            source: self.source,
        })
    }
}

struct ObsoleteValueVisitor<'a> {
    source: &'a str,
}

impl<'de> Visitor<'de> for ObsoleteValueVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "obsolete Orca project option {} as a string or array",
            self.source
        )
    }

    fn visit_borrowed_str<E>(self, _value: &'de str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_str<E>(self, _value: &str) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_string<E>(self, _value: String) -> Result<Self::Value, E> {
        Ok(())
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        while sequence.next_element::<IgnoredAny>()?.is_some() {}
        Ok(())
    }
}

struct LegacyValueSeed<'a> {
    builder: &'a mut ProjectSettingsBuilder,
    rule: &'static LegacyRule,
}

impl<'de> DeserializeSeed<'de> for LegacyValueSeed<'_> {
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(LegacyValueVisitor {
            builder: self.builder,
            rule: self.rule,
        })
    }
}

struct LegacyValueVisitor<'a> {
    builder: &'a mut ProjectSettingsBuilder,
    rule: &'static LegacyRule,
}

impl<'de> Visitor<'de> for LegacyValueVisitor<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "legacy Orca project option {} as a string or array",
            self.rule.source
        )
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(value)
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        let triggered_effect = self
            .rule
            .json_effect
            .filter(|effect| effect.triggers.contains(&value));
        apply_outcome(
            self.builder,
            self.rule,
            transform_lexical(self.rule, value),
            lexical::ValueOrigin::Scalar,
        )?;
        if let Some(effect) = triggered_effect {
            self.builder.schedule_json_effect(effect);
        }
        Ok(())
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: Error,
    {
        self.visit_str(&value)
    }

    fn visit_seq<A>(self, sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let outcome = transform_json_array(self.rule, SeqAccessDeserializer::new(sequence));
        apply_outcome(
            self.builder,
            self.rule,
            outcome,
            lexical::ValueOrigin::Array,
        )
    }
}

fn apply_outcome<E>(
    builder: &mut ProjectSettingsBuilder,
    rule: &'static LegacyRule,
    outcome: LegacyOutcome,
    origin: lexical::ValueOrigin,
) -> Result<(), E>
where
    E: Error,
{
    match outcome {
        LegacyOutcome::Assign { target, value } => {
            lexical::assign(builder, rule, target, value, origin)
        }
        LegacyOutcome::Consume => Ok(()),
        LegacyOutcome::Deferred { source, .. } => Err(E::custom(format_args!(
            "unsupported deferred Orca project option {source}"
        ))),
        LegacyOutcome::Error(LegacyTransformError::InvalidArrayValue { source }) => Err(E::custom(
            format_args!("invalid legacy Orca project array option {source}"),
        )),
    }
}
