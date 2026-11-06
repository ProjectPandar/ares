use super::compare;

#[tokio::test]
async fn stock_validation_01() {
    compare("Anycubic", "Anycubic Kobra S1 Max 0.25 nozzle", "filament_flush_temp: nil not in range [0, 1500]\nretraction_distances_when_cut: 0 not in range [10, 18]", true).await;
}

#[tokio::test]
async fn stock_validation_02() {
    compare(
        "Anycubic",
        "Anycubic Kobra S1 Max 0.4 nozzle",
        "retraction_distances_when_cut: 0 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_03() {
    compare("Anycubic", "Anycubic Kobra S1 Max 0.6 nozzle", "filament_flush_temp: nil not in range [0, 1500]\nretraction_distances_when_cut: 0 not in range [10, 18]", true).await;
}

#[tokio::test]
async fn stock_validation_04() {
    compare("Anycubic", "Anycubic Kobra S1 Max 0.8 nozzle", "filament_flush_temp: nil not in range [0, 1500]\nretraction_distances_when_cut: 0 not in range [10, 18]", true).await;
}

#[tokio::test]
async fn stock_validation_05() {
    compare(
        "Anycubic",
        "Anycubic Kobra X 0.4 nozzle",
        "retraction_distances_when_cut: 0 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_06() {
    compare(
        "Creality",
        "Creality CR-10 SE 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_07() {
    compare(
        "Creality",
        "Creality Ender-3 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_08() {
    compare(
        "Creality",
        "Creality Ender-3 Pro 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_09() {
    compare(
        "Creality",
        "Creality Ender-3 S1 Plus 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_10() {
    compare(
        "Creality",
        "Creality Ender-3 V3 KE 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_11() {
    compare(
        "Creality",
        "Creality Ender-3 V3 SE 0.2 nozzle",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_12() {
    compare(
        "Creality",
        "Creality K2 0.4 nozzle",
        "retraction_distances_when_cut: 30 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_13() {
    compare(
        "Creality",
        "Creality SPARKX i7 0.2 nozzle",
        "retraction_distances_when_cut: 28 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_14() {
    compare(
        "Creality",
        "Creality SPARKX i7 0.4 nozzle",
        "retraction_distances_when_cut: 28 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_15() {
    compare(
        "Creality",
        "Creality SPARKX i7 0.6 nozzle",
        "retraction_distances_when_cut: 28 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_16() {
    compare(
        "Creality",
        "Creality SPARKX i7 0.8 nozzle",
        "retraction_distances_when_cut: 28 not in range [10, 18]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_17() {
    compare(
        "LONGER",
        "LONGER LK10 (0.2 nozzle)",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_18() {
    compare(
        "LONGER",
        "LONGER LK10 Plus (0.2 nozzle)",
        "Line width too small",
        false,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_19() {
    compare(
        "RH3D",
        "E3NG v1.2S - 0.2 nozzle",
        "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_20() {
    compare(
        "RH3D",
        "E3NG v1.2S - 0.3 nozzle",
        "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_21() {
    compare(
        "RH3D",
        "E3NG v1.2S - 0.4 nozzle",
        "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_22() {
    compare(
        "RH3D",
        "E3NG v1.2S - 0.5 nozzle",
        "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_23() {
    compare(
        "RH3D",
        "E3NG v1.2S - 0.6 nozzle",
        "use_firmware_retraction: --use-firmware-retraction is not compatible with --wipe",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_24() {
    compare(
        "SeeMeCNC",
        "SeeMeCNC BOSSdelta 500 0521 0.4 nozzle",
        "extruder_printable_height: 2100 not in range [0, 1000]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_25() {
    compare(
        "SeeMeCNC",
        "SeeMeCNC BOSSdelta 500 0521 0.5 nozzle",
        "extruder_printable_height: 2100 not in range [0, 1000]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_26() {
    compare(
        "SeeMeCNC",
        "SeeMeCNC BOSSdelta 500 0521 0.7 nozzle",
        "extruder_printable_height: 2100 not in range [0, 1000]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_27() {
    compare(
        "SeeMeCNC",
        "SeeMeCNC BOSSdelta 500 0521 1.0 nozzle",
        "extruder_printable_height: 2100 not in range [0, 1000]",
        true,
    )
    .await;
}

#[tokio::test]
async fn stock_validation_28() {
    compare(
        "Snapmaker",
        "Snapmaker U1 (0.4+0.6 nozzle)",
        "bridge_line_width: Bridge line width must not exceed nozzle diameter: 0.600000",
        true,
    )
    .await;
}
