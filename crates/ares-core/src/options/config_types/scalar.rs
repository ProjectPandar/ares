use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, IntoDeserializer, Visitor},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrcaFloat(pub f64);

impl Serialize for OrcaFloat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.0.is_finite() {
            return Err(serde::ser::Error::custom(
                "Orca numeric value must be finite",
            ));
        }
        serializer.serialize_str(&format_number(self.0))
    }
}

impl<'de> Deserialize<'de> for OrcaFloat {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FloatVisitor;

        impl<'de> Visitor<'de> for FloatVisitor {
            type Value = OrcaFloat;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a finite Orca float")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                finite(value).map(OrcaFloat).map_err(E::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                value
                    .trim()
                    .parse::<f64>()
                    .map_err(E::custom)
                    .and_then(|value| self.visit_f64(value))
            }
        }

        deserializer.deserialize_any(FloatVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Millimeters(pub f64);

impl Serialize for Millimeters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.0.is_finite() {
            return Err(serde::ser::Error::custom(
                "Orca numeric value must be finite",
            ));
        }
        OrcaFloat(self.0).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Millimeters {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        OrcaFloat::deserialize(deserializer).map(|value| Self(value.0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Percent(pub f64);

impl Serialize for Percent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if !self.0.is_finite() {
            return Err(serde::ser::Error::custom(
                "Orca numeric value must be finite",
            ));
        }
        serializer.serialize_str(&format!("{}%", format_number(self.0)))
    }
}

impl<'de> Deserialize<'de> for Percent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PercentVisitor;

        impl<'de> Visitor<'de> for PercentVisitor {
            type Value = Percent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a finite Orca percentage")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                finite(value).map(Percent).map_err(E::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let value = value.trim();
                let value = value.strip_suffix('%').unwrap_or(value).trim();
                value
                    .parse::<f64>()
                    .map_err(E::custom)
                    .and_then(|value| self.visit_f64(value))
            }
        }

        deserializer.deserialize_any(PercentVisitor)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FloatOrPercent {
    Float(f64),
    Percent(Percent),
}

impl FloatOrPercent {
    pub(crate) fn is_non_positive(self) -> bool {
        match self {
            Self::Float(value) => value <= 0.0,
            Self::Percent(value) => value.0 <= 0.0,
        }
    }
}

impl Serialize for FloatOrPercent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Float(value) => OrcaFloat(*value).serialize(serializer),
            Self::Percent(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for FloatOrPercent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UnionVisitor;

        impl<'de> Visitor<'de> for UnionVisitor {
            type Value = FloatOrPercent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a finite float or percentage")
            }

            fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                finite(value).map(FloatOrPercent::Float).map_err(E::custom)
            }

            fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                self.visit_f64(value as f64)
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                parse_float_or_percent(value)
            }
        }

        deserializer.deserialize_any(UnionVisitor)
    }
}

fn parse_float_or_percent<E>(value: &str) -> Result<FloatOrPercent, E>
where
    E: Error,
{
    let value = value.trim();
    if value.ends_with('%') {
        Percent::deserialize(value.into_deserializer()).map(FloatOrPercent::Percent)
    } else {
        OrcaFloat::deserialize(value.into_deserializer())
            .map(|value| FloatOrPercent::Float(value.0))
    }
}

fn finite(value: f64) -> Result<f64, &'static str> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err("Orca numeric value must be finite")
    }
}

pub(crate) fn format_number(value: f64) -> String {
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0".to_owned()
        } else {
            "0".to_owned()
        };
    }
    let scientific = format!("{value:.5e}");
    let (mantissa, raw_exponent) = scientific.split_once('e').unwrap();
    let exponent = raw_exponent.parse::<i32>().unwrap();
    if !(-4..6).contains(&exponent) {
        let mut mantissa = mantissa.to_owned();
        trim_fraction(&mut mantissa);
        format!("{mantissa}e{exponent:+03}")
    } else {
        let decimals = (5 - exponent).max(0) as usize;
        let mut output = format!("{value:.decimals$}");
        trim_fraction(&mut output);
        output
    }
}

fn trim_fraction(value: &mut String) {
    if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
}
