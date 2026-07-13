use crate::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, Percent, ProcessBrimType,
    ProcessExtraBridgeLayer, ProcessGapFillTarget, ProcessInfillPattern,
    ProcessInternalBridgeFilter, ProcessPerimeterGenerator, ProcessSeamPosition,
    ProcessSlicingMode, ProcessSupportBasePattern, ProcessSupportInterfacePattern,
    ProcessSupportStyle, ProcessSupportType,
};

use super::{ObjectOptionOverrides, ObjectOptions, ProcessObjectSourceOptions};
use super::super::process_object_source::expected::DECLARATION_ORDER;

mod cases;

use cases::object_projection_cases;

fn project(
    base: &ProcessObjectSourceOptions,
    overrides: &ObjectOptionOverrides,
) -> ObjectOptions {
    ObjectOptions::overlay(base, overrides)
}

macro_rules! define_projection_oracle {
    ($($field:ident: $ty:ty => ($raw:expr, $alternate:expr)),* $(,)?) => {
        const PROJECTION_ORDER: [&str; 126] = [$(stringify!($field)),*];

        fn all_non_default_base() -> ProcessObjectSourceOptions {
            ProcessObjectSourceOptions {
                $($field: $alternate),*
            }
        }

        fn expected_from_base(base: &ProcessObjectSourceOptions) -> ObjectOptions {
            ObjectOptions {
                $($field: base.$field),*
            }
        }

        fn assert_all_fields(actual: &ObjectOptions, expected: &ObjectOptions) {
            $(assert_eq!(actual.$field, expected.$field, stringify!($field));)*
        }

        fn present_fields(overrides: &ObjectOptionOverrides) -> Vec<&'static str> {
            let mut present = Vec::new();
            $(
                if overrides.$field.is_some() {
                    present.push(stringify!($field));
                }
            )*
            present
        }

        fn assert_raw_and_alternate_are_distinct() {
            $(
                let raw: $ty = $raw;
                let alternate: $ty = $alternate;
                assert_ne!(raw, alternate, stringify!($field));
            )*
        }

        #[test]
        fn object_options_projection_absent_overrides_inherit_all_126_fields() {
            assert_eq!(PROJECTION_ORDER, DECLARATION_ORDER);
            assert_raw_and_alternate_are_distinct();
            let base = all_non_default_base();
            let overrides = ObjectOptionOverrides::default();
            let expected = expected_from_base(&base);
            let actual = project(&base, &overrides);

            assert!(present_fields(&overrides).is_empty());
            assert_all_fields(&actual, &expected);
            assert_eq!(actual, expected);
        }

        #[test]
        fn object_options_projection_each_present_raw_default_replaces_only_it() {
            assert_eq!(PROJECTION_ORDER, DECLARATION_ORDER);
            $(
                {
                    let base = all_non_default_base();
                    let raw: $ty = $raw;
                    let overrides = ObjectOptionOverrides {
                        $field: Some(raw),
                        ..Default::default()
                    };
                    assert_eq!(present_fields(&overrides), [stringify!($field)]);

                    let mut expected = expected_from_base(&base);
                    expected.$field = raw;
                    let actual = project(&base, &overrides);

                    assert_eq!(actual.$field, raw, stringify!($field));
                    assert_ne!(actual.$field, base.$field, stringify!($field));
                    assert_all_fields(&actual, &expected);
                    assert_eq!(actual, expected, stringify!($field));
                }
            )*
        }
    };
}

object_projection_cases!(define_projection_oracle);

#[test]
fn object_options_projection_preserves_high_support_filaments_before_clamp() {
    let representative_extruder_count = 2;
    let base = all_non_default_base();
    let overrides = ObjectOptionOverrides {
        support_filament: Some(OrcaInt(7)),
        support_interface_filament: Some(OrcaInt(8)),
        ..Default::default()
    };
    assert!(overrides.support_filament.unwrap().0 > representative_extruder_count);
    assert!(overrides.support_interface_filament.unwrap().0 > representative_extruder_count);

    let mut expected = expected_from_base(&base);
    expected.support_filament = OrcaInt(7);
    expected.support_interface_filament = OrcaInt(8);
    let actual = project(&base, &overrides);

    assert_eq!(
        present_fields(&overrides),
        ["support_filament", "support_interface_filament"]
    );
    assert_all_fields(&actual, &expected);
    assert_eq!(actual, expected);
}
