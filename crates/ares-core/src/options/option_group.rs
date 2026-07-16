use super::{
    CsvTable, OrcaBools, OrcaFloats, OrcaInts, OrcaPercents, OrcaStrings, RammingParameters,
    SpaceTuple, VariantStride,
};

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
