use std::fmt;

use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{Error, SeqAccess, Visitor},
    ser::SerializeSeq,
};

use super::scalar::format_number;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point2d {
    pub x: f64,
    pub y: f64,
}

impl Point2d {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl Serialize for Point2d {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        validate_point(*self).map_err(serde::ser::Error::custom)?;
        serializer.serialize_str(&format_point(*self, ','))
    }
}

impl<'de> Deserialize<'de> for Point2d {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PointVisitor;

        impl<'de> Visitor<'de> for PointVisitor {
            type Value = Point2d;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an Orca point x,y or xXy")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                parse_point(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(PointVisitor)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Point2dList(pub Vec<Point2d>);

impl Serialize for Point2dList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        for point in &self.0 {
            validate_point(*point).map_err(serde::ser::Error::custom)?;
        }
        let values = self
            .0
            .iter()
            .map(|point| format_point(*point, 'x'))
            .collect::<Vec<_>>();
        values.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Point2dList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_strings(deserializer, parse_vector_point).map(Self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Point2dGroups(pub Vec<Vec<Point2d>>);

impl Serialize for Point2dGroups {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.0.len()))?;
        for group in &self.0 {
            for point in group {
                validate_point(*point).map_err(serde::ser::Error::custom)?;
            }
            let value = group
                .iter()
                .map(|point| format_point(*point, 'x'))
                .collect::<Vec<_>>()
                .join(",");
            sequence.serialize_element(&value)?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for Point2dGroups {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserialize_strings(deserializer, |group| {
            if group.is_empty() {
                Ok(Vec::new())
            } else {
                group.split(',').map(parse_point).collect()
            }
        })
        .map(Self)
    }
}

fn parse_point(value: &str) -> Result<Point2d, &'static str> {
    let separator = if value.contains(',') { ',' } else { 'x' };
    let mut coordinates = value.split(separator);
    let x = parse_coordinate(coordinates.next())?;
    let y = parse_coordinate(coordinates.next())?;
    if coordinates.next().is_some() {
        return Err("Orca point must have exactly two coordinates");
    }
    Ok(Point2d { x, y })
}

fn parse_vector_point(value: &str) -> Result<Point2d, &'static str> {
    if !value.contains('x') || value.contains(',') {
        return Err("Orca point vector elements must use x separators");
    }
    parse_point(value)
}

fn parse_coordinate(value: Option<&str>) -> Result<f64, &'static str> {
    let value = value
        .ok_or("Orca point must have exactly two coordinates")?
        .trim()
        .parse::<f64>()
        .map_err(|_| "Orca point contains an invalid coordinate")?;
    if value.is_finite() {
        Ok(value)
    } else {
        Err("Orca point coordinate must be finite")
    }
}

fn format_point(point: Point2d, separator: char) -> String {
    format!(
        "{}{}{}",
        format_number(point.x),
        separator,
        format_number(point.y)
    )
}

fn validate_point(point: Point2d) -> Result<(), &'static str> {
    if point.x.is_finite() && point.y.is_finite() {
        Ok(())
    } else {
        Err("Orca point coordinates must be finite")
    }
}

fn deserialize_strings<'de, D, T>(
    deserializer: D,
    parse: impl Fn(&str) -> Result<T, &'static str>,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringSequenceVisitor<F>(F);

    impl<'de, T, F> Visitor<'de> for StringSequenceVisitor<F>
    where
        F: Fn(&str) -> Result<T, &'static str>,
    {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an array of Orca strings")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::with_capacity(sequence.size_hint().unwrap_or(0));
            while let Some(value) = sequence.next_element::<String>()? {
                values.push((self.0)(&value).map_err(A::Error::custom)?);
            }
            Ok(values)
        }
    }

    deserializer.deserialize_seq(StringSequenceVisitor(parse))
}
