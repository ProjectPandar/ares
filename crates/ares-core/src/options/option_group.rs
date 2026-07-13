#[allow(unused_macros)]
macro_rules! declare_option_group {
    (
        $visibility:vis struct $group:ident, $builder:ident {
            $($field:ident => $key:literal: $ty:ty = $default:expr),* $(,)?
        }
    ) => {
        #[derive(Clone, Debug, PartialEq)]
        $visibility struct $group {
            $($visibility $field: $ty),*
        }

        #[derive(Default)]
        pub(crate) struct $builder {
            $($field: Option<$ty>),*
        }

        impl $builder {
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
