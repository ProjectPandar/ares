use super::{
    CsvTable, Nullable, OrcaBools, OrcaFloat, OrcaFloats, OrcaInts, OrcaPercents, OrcaStrings,
    Percent, RammingParameters, SpaceTuple, VariantStride,
};
use crate::SliceError;

pub(crate) trait VariantVector {
    type Item: Clone;

    fn variant_values(&self) -> &[Self::Item];
    fn variant_values_mut(&mut self) -> &mut Vec<Self::Item>;
}

impl<T: Clone> VariantVector for Vec<T> {
    type Item = T;

    fn variant_values(&self) -> &[Self::Item] {
        self
    }

    fn variant_values_mut(&mut self) -> &mut Vec<Self::Item> {
        self
    }
}

macro_rules! impl_variant_vector {
    ($($ty:ty => $item:ty),+ $(,)?) => {
        $(
            impl VariantVector for $ty {
                type Item = $item;

                fn variant_values(&self) -> &[Self::Item] {
                    &self.0
                }

                fn variant_values_mut(&mut self) -> &mut Vec<Self::Item> {
                    &mut self.0
                }
            }
        )+
    };
}

impl_variant_vector!(
    OrcaFloats => OrcaFloat,
    OrcaInts => super::OrcaInt,
    OrcaBools => super::OrcaBool,
    SpaceTuple => String,
    VariantStride => String,
);

pub(crate) fn normalize_root_variant_vector<V>(
    values: &mut V,
    defaults: &V,
    target: usize,
    key: &'static str,
    reset: impl FnOnce(&[V::Item]) -> bool,
) -> Result<(), SliceError>
where
    V: VariantVector,
{
    if target == 0 {
        values.variant_values_mut().clear();
        return Ok(());
    }
    if reset(values.variant_values()) {
        let defaults = defaults.variant_values();
        let values = values.variant_values_mut();
        values.clear();
        values.extend_from_slice(defaults);
    }
    normalize_variant_vector(values, target, key)
}

pub(crate) fn normalize_present_variant_vector<V>(
    values: &mut V,
    target: usize,
    key: &'static str,
) -> Result<(), SliceError>
where
    V: VariantVector,
{
    if target == 0 {
        values.variant_values_mut().clear();
        return Ok(());
    }
    normalize_variant_vector(values, target, key)
}

fn normalize_variant_vector<V>(
    values: &mut V,
    target: usize,
    key: &'static str,
) -> Result<(), SliceError>
where
    V: VariantVector,
{
    let values = values.variant_values_mut();
    let first = values
        .first()
        .cloned()
        .ok_or_else(|| SliceError::InvalidInput(format!("{key} must not be empty")))?;
    values.resize(target, first);
    Ok(())
}

pub(crate) fn exact_variant_vectors_equal<T: PartialEq>(source: &[T], child: &[T]) -> bool {
    source == child
}

pub(crate) fn nullable_float_variant_vectors_equal(
    source: &[Nullable<OrcaFloat>],
    child: &[Nullable<OrcaFloat>],
) -> bool {
    nullable_variant_vectors_equal(source, child, |value| value.0)
}

pub(crate) fn nullable_percent_variant_vectors_equal(
    source: &[Nullable<Percent>],
    child: &[Nullable<Percent>],
) -> bool {
    nullable_variant_vectors_equal(source, child, |value| value.0)
}

fn nullable_variant_vectors_equal<T>(
    source: &[Nullable<T>],
    child: &[Nullable<T>],
    value: impl Fn(&T) -> f64,
) -> bool {
    const EPSILON: f64 = 1e-4;

    source.len() == child.len()
        && source
            .iter()
            .zip(child)
            .all(|(source, child)| match (source, child) {
                (Nullable::Nil, Nullable::Nil) => true,
                (Nullable::Value(source), Nullable::Value(child)) => {
                    (value(source) - value(child)).abs() < EPSILON
                }
                _ => false,
            })
}

pub(crate) fn apply_variant_slots<T: Clone>(
    source: &mut Vec<T>,
    child: &[T],
    mapping: &[Option<usize>],
    key: &'static str,
    (equal, replace): (impl FnOnce(&[T], &[T]) -> bool, impl Fn(&T) -> bool),
) -> Result<(), SliceError> {
    if equal(source, child) {
        return Ok(());
    }
    if source.len() != mapping.len() {
        source.clear();
        source.extend_from_slice(child);
        return Ok(());
    }
    for (source, child_index) in source
        .iter_mut()
        .zip(mapping)
        .filter_map(|(source, index)| index.map(|index| (source, index)))
    {
        let child = child.get(child_index).ok_or_else(|| {
            SliceError::InvalidInput(format!("{key} is missing variant slot {child_index}"))
        })?;
        if replace(child) {
            source.clone_from(child);
        }
    }
    Ok(())
}

pub(crate) trait OverlayOptionGroup {
    fn overlay(&mut self, child: Self);
}

pub(crate) trait AppendOptionValue {
    fn append_value(&mut self, child: Self);
}

impl<T> AppendOptionValue for Vec<T> {
    fn append_value(&mut self, mut child: Self) {
        self.append(&mut child);
    }
}

macro_rules! impl_append_option_value {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl AppendOptionValue for $ty {
                fn append_value(&mut self, mut child: Self) {
                    self.0.append(&mut child.0);
                }
            }
        )+
    };
}

impl_append_option_value!(
    CsvTable,
    OrcaBools,
    OrcaFloats,
    OrcaInts,
    OrcaPercents,
    OrcaStrings,
    RammingParameters,
    SpaceTuple,
    VariantStride,
);

#[allow(unused_macros)]
macro_rules! declare_option_group {
    (
        append $visibility:vis struct $group:ident, $builder:ident {
            $($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?
        }
    ) => {
        declare_option_group! {
            @declare $visibility struct $group, $builder {
                $($field => $key: $ty = $default),*
            }
        }

        impl $group {
            pub(crate) fn append(&mut self, child: Self) {
                $(
                    $crate::options::option_group::AppendOptionValue::append_value(
                        &mut self.$field,
                        child.$field,
                    );
                )*
            }
        }

        impl $builder {
            pub(crate) fn apply_present(self, target: &mut $group) {
                $(
                    if let Some(value) = self.$field {
                        target.$field = value;
                    }
                )*
            }
        }
    };
    (
        $visibility:vis struct $group:ident, $builder:ident {
            $($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?
        }
    ) => {
        declare_option_group! {
            @declare $visibility struct $group, $builder {
                $($field => $key: $ty = $default),*
            }
        }
    };
    (
        @declare $visibility:vis struct $group:ident, $builder:ident {
            $($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        $visibility struct $group {
            $($visibility $field: $ty),*
        }

        #[derive(Clone, Default, PartialEq)]
        pub(crate) struct $builder {
            $($field: Option<$ty>),*
        }

        impl $builder {
            #[allow(dead_code)]
            pub(crate) fn is_known_field(key: &str) -> bool {
                matches!(key, $($key)|*)
            }

            pub(crate) fn deserialize_known_field<'de, A>(
                &mut self,
                key: &str,
                map: &mut A,
            ) -> Result<bool, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                match key {
                    $(
                        $key => {
                            if self.$field.is_some() {
                                return Err(serde::de::Error::custom(concat!(
                                    "duplicate Orca option ", $key
                                )));
                            }
                            self.$field = Some(map.next_value::<$ty>().map_err(|error| {
                                serde::de::Error::custom(format_args!(
                                    concat!("invalid Orca option ", $key, ": {}"),
                                    error
                                ))
                            })?);
                            Ok(true)
                        }
                    ),*
                    _ => Ok(false),
                }
            }

            #[allow(dead_code)]
            pub(crate) fn deserialize_known_value<'de, D>(
                &mut self,
                key: &str,
                deserializer: D,
            ) -> Result<bool, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                match key {
                    $(
                        $key => {
                            if self.$field.is_some() {
                                return Err(serde::de::Error::custom(concat!(
                                    "duplicate Orca option ", $key
                                )));
                            }
                            self.$field = Some(<$ty as serde::Deserialize>::deserialize(
                                deserializer,
                            ).map_err(|error| {
                                serde::de::Error::custom(format_args!(
                                    concat!("invalid Orca option ", $key, ": {}"),
                                    error
                                ))
                            })?);
                            Ok(true)
                        }
                    ),*
                    _ => Ok(false),
                }
            }
        }

        impl $crate::options::option_group::OverlayOptionGroup for $builder {
            fn overlay(&mut self, child: Self) {
                $(
                    if let Some(value) = child.$field {
                        self.$field = Some(value);
                    }
                )*
            }
        }

        impl $builder {
            pub(crate) fn resolve(self) -> $group {
                $group {
                    $($field: self.$field.unwrap_or_else(|| $default)),*
                }
            }
        }
    };
}

#[allow(unused_imports)]
pub(crate) use declare_option_group;
