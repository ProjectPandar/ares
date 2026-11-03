//! Config.hpp ConfigOptionSingle<T>/ConfigOptionVector<T>::operator==:
//! compare option-owned values, not JSON or wire spellings.

use serde::de::DeserializeOwned;

use crate::{
    options::{
        FloatOrPercent, Nullable, OrcaBool, OrcaBools, OrcaBoolsNullable, OrcaFloat,
        OrcaFloatOrPercents, OrcaFloats, OrcaFloatsNullable, OrcaInt, OrcaInts, OrcaPercents,
        OrcaStrings, Percent, Point2d, Point2dGroups, Point2dList, registry::OptionValueKind,
    },
    project::raw_settings::RawConfigValue,
};

pub(super) fn equal(
    kind: OptionValueKind,
    left: &RawConfigValue,
    right: &RawConfigValue,
) -> Result<bool, String> {
    match kind {
        OptionValueKind::Float => compare::<OrcaFloat>(left, right),
        OptionValueKind::FloatOrPercent => compare::<FloatOrPercent>(left, right),
        OptionValueKind::FloatsOrPercents => compare::<OrcaFloatOrPercents>(left, right),
        OptionValueKind::Percent => compare::<Percent>(left, right),
        OptionValueKind::Percents => compare::<OrcaPercents>(left, right),
        OptionValueKind::PercentsNullable => compare::<Vec<Nullable<Percent>>>(left, right),
        OptionValueKind::Int => compare::<OrcaInt>(left, right),
        OptionValueKind::Bool => compare::<OrcaBool>(left, right),
        OptionValueKind::Bools => compare::<OrcaBools>(left, right),
        OptionValueKind::BoolsNullable => compare::<OrcaBoolsNullable>(left, right),
        OptionValueKind::Enum | OptionValueKind::String => compare::<String>(left, right),
        OptionValueKind::Enums | OptionValueKind::Strings => compare::<OrcaStrings>(left, right),
        OptionValueKind::EnumsNullable => compare::<Vec<Nullable<String>>>(left, right),
        OptionValueKind::Floats => compare::<OrcaFloats>(left, right),
        OptionValueKind::FloatsNullable => compare::<OrcaFloatsNullable>(left, right),
        OptionValueKind::Ints => compare::<OrcaInts>(left, right),
        OptionValueKind::IntsNullable => compare::<Vec<Nullable<OrcaInt>>>(left, right),
        OptionValueKind::Point => compare::<Point2d>(left, right),
        OptionValueKind::Points => compare::<Point2dList>(left, right),
        OptionValueKind::PointsGroups => compare::<Point2dGroups>(left, right),
    }
}

fn compare<T: DeserializeOwned + PartialEq>(
    left: &RawConfigValue,
    right: &RawConfigValue,
) -> Result<bool, String> {
    let left = T::deserialize(left.clone()).map_err(|error| error.to_string())?;
    let right = T::deserialize(right.clone()).map_err(|error| error.to_string())?;
    Ok(left == right)
}
