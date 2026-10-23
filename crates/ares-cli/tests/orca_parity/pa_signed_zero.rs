//! `GCodeWriter.cpp:370-391` admits signed zero and preserves its streamed sign.

#[test]
fn negative_zero_fixed_pa_keeps_sign() {
    check(
        "negative-zero",
        include_bytes!("../../../../tests/parity/pa-signed-zero/negative-zero.3mf"),
        "752f7ddf2c430856d90877a5e902100338fb9689ce62884cf7c9bec27c7e6fab",
        Some(true),
    );
}

#[test]
fn positive_zero_fixed_pa_control_stays_byte_equal() {
    check(
        "positive-zero",
        include_bytes!("../../../../tests/parity/pa-signed-zero/positive-zero.3mf"),
        "67ffaf0ffe1409ebf2a264fe3b8b087b84ca61cc035db12a529be3077e55553b",
        Some(false),
    );
}

#[test]
fn disabled_fixed_pa_stock_control_stays_byte_equal() {
    check(
        "stock-control",
        include_bytes!("../../../../tests/parity/pa-signed-zero/stock-control.3mf"),
        "a360b2d4d71a140a197726b8ab4c8c8b370ad2e17957fa742da2a2346f9336ae",
        None,
    );
}

fn check(label: &str, project: &[u8], sha: &str, negative: Option<bool>) {
    crate::runner::native_stream_case::check(
        (
            "pa-signed-zero",
            &[
                "GCode.cpp:3155-3161::initial-filament pressure advance",
                "GCodeWriter.cpp:370-391::set_pressure_advance",
            ],
        ),
        label,
        project,
        sha,
        (false, |reference, _| {
            let command = reference
                .native
                .lines()
                .find(|line| line.contains("; Override pressure advance value"));
            if let Some(negative) = negative {
                let value: f64 = command
                    .unwrap()
                    .strip_prefix("M900 K")
                    .unwrap()
                    .split_once(';')
                    .unwrap()
                    .0
                    .parse()
                    .unwrap();
                assert_eq!(value.classify(), std::num::FpCategory::Zero);
                assert_eq!(value.is_sign_negative(), negative);
            } else {
                assert!(
                    command.is_none(),
                    "Native disabled fixed PA emits no override"
                );
            }
        }),
    );
}
