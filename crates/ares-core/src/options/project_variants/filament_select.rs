//! Zero-initialized payload selection (`PrintConfig.cpp:9879-10015`).

use crate::{
    SliceError,
    options::{Nullable, OrcaBool, OrcaFloat, OrcaInt, Percent, RetractLiftEnforce, ZHopType},
};

pub(super) trait FilamentZero: Clone {
    fn zero() -> Self;
}

macro_rules! zero_value {
    ($($ty:ty => $value:expr),+ $(,)?) => {
        $(impl FilamentZero for $ty {
            fn zero() -> Self {
                $value
            }
        })+
    };
}

zero_value!(
    String => String::new(),
    OrcaBool => OrcaBool(false),
    OrcaFloat => OrcaFloat(0.0),
    OrcaInt => OrcaInt(0),
    Percent => Percent(0.0),
    ZHopType => ZHopType::Auto,
    RetractLiftEnforce => RetractLiftEnforce::AllSurfaces,
);

impl<T: FilamentZero> FilamentZero for Nullable<T> {
    fn zero() -> Self {
        Self::Value(T::zero())
    }
}

pub(super) fn select_filament<T: FilamentZero>(
    values: &[T],
    indices: &[usize],
    key: &str,
) -> Result<Vec<T>, SliceError> {
    if values.is_empty() {
        return Err(SliceError::InvalidInput(format!(
            "invalid Orca option {key}"
        )));
    }
    Ok(indices
        .iter()
        .map(|&index| values.get(index).cloned().unwrap_or_else(T::zero))
        .collect())
}
