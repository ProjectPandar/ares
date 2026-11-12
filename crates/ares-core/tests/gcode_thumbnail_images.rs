use ares_core::ThumbnailData;

#[path = "gcode_thumbnail_images/project.rs"]
mod project;
#[path = "gcode_thumbnail_images/samples.rs"]
mod samples;
#[path = "gcode_thumbnail_images/support.rs"]
mod support;

use support::{Control, Expected};

#[test]
fn caller_thumbnail_data_rejects_invalid_dimensions_and_rgba_lengths() {
    assert!(ThumbnailData::new(0, 24, Vec::new()).is_err());
    assert!(ThumbnailData::new(32, 0, Vec::new()).is_err());
    assert!(ThumbnailData::new(32, 24, vec![0; 32 * 24 * 4 - 1]).is_err());
    assert!(ThumbnailData::new(32, 24, vec![0; 32 * 24 * 4 + 1]).is_err());
    assert!(ThumbnailData::new(u32::MAX, u32::MAX, vec![0; 4]).is_err());
    assert!(ThumbnailData::new(1_u32 << 31, 1_u32 << 31, Vec::new()).is_err());
}

#[tokio::test]
async fn caller_rgba_png_matches_native_export_through_public_project_slice() {
    support::codec("PNG").await;
}

#[tokio::test]
async fn caller_rgba_jpg_matches_native_export_through_public_project_slice() {
    support::codec("JPG").await;
}

#[tokio::test]
async fn caller_rgba_qoi_matches_native_export_through_public_project_slice() {
    support::codec("QOI").await;
}

#[tokio::test]
async fn caller_rgba_btt_tft_matches_native_export_through_public_project_slice() {
    support::codec("BTT_TFT").await;
}

#[tokio::test]
async fn caller_rgba_colpic_matches_native_export_through_public_project_slice() {
    support::codec("COLPIC").await;
}

#[tokio::test]
async fn colpic_at_size_cap_with_palette_overflow_matches_native() {
    support::check(
        "colpic-at-cap",
        "512x37/COLPIC",
        vec![samples::high_color(512, 37)],
        Expected::Blocks,
    )
    .await;
}

#[tokio::test]
async fn colpic_width_capping_with_palette_overflow_matches_native() {
    support::check(
        "colpic-width-cap",
        "513x37/COLPIC",
        vec![samples::high_color(513, 37)],
        Expected::Blocks,
    )
    .await;
}

#[tokio::test]
async fn colpic_height_capping_with_palette_overflow_matches_native() {
    support::check(
        "colpic-height-cap",
        "37x513/COLPIC",
        vec![samples::high_color(37, 513)],
        Expected::Blocks,
    )
    .await;
}

#[tokio::test]
async fn mixed_and_repeated_definitions_keep_native_order_tags_and_btt_terminators() {
    for definitions in [
        "32x24/PNG, 32x24/JPG, 32x24/QOI, 32x24/BTT_TFT, 32x24/COLPIC, 32x24/PNG, 32x24/COLPIC",
        "32x24/PNG, 32x24/BTT_TFT",
        "32x24/BTT_TFT, 32x24/PNG",
    ] {
        support::check(
            "mixed-repeated",
            definitions,
            vec![samples::asymmetric()],
            Expected::Blocks,
        )
        .await;
    }
}

#[tokio::test]
async fn fractional_requested_size_emits_actual_frame_dimensions_like_native() {
    let mut frame = samples::asymmetric();
    frame.size = [32.75, 24.5];
    support::check(
        "fractional",
        "32.75x24.5/PNG",
        vec![frame],
        Expected::Blocks,
    )
    .await;
}

#[tokio::test]
async fn missing_frames_and_empty_definitions_emit_no_blocks_or_other_changes() {
    support::check(
        "missing",
        "64x48/PNG",
        vec![samples::asymmetric()],
        Expected::Empty,
    )
    .await;
    support::check(
        "subpixel-missing",
        "0.25x0.75/PNG",
        vec![samples::asymmetric()],
        Expected::Empty,
    )
    .await;
    support::check("empty-frames", "32x24/PNG", Vec::new(), Expected::Empty).await;
    support::check(
        "empty-definitions",
        "",
        vec![samples::asymmetric()],
        Expected::Empty,
    )
    .await;
}

#[tokio::test]
async fn supplied_images_preserve_reachable_thumbnail_load_errors() {
    for definitions in [
        "bad",
        "0x24/PNG",
        "1000x24/PNG",
        "32x24/UNKNOWN",
        "bad, 1000x24/PNG",
        "bad, 32x24/UNKNOWN",
        "1000x24/UNKNOWN",
        "bad, 1000x24/UNKNOWN",
    ] {
        support::unchanged(
            project::ENDER3,
            definitions,
            vec![samples::asymmetric()],
            Control::ExistingLoadError,
        )
        .await;
    }
}

#[tokio::test]
async fn empty_caller_images_preserve_complete_stream_without_new_validation() {
    for definitions in [
        "32x24/PNG",
        "32x24/JPG",
        "32x24/QOI",
        "32x24/BTT_TFT",
        "32x24/COLPIC",
        "",
    ] {
        support::unchanged(
            project::ENDER3,
            definitions,
            Vec::new(),
            Control::CompleteStream,
        )
        .await;
    }
    support::unchanged(
        project::ENDER3,
        "bad, 1000x24/UNKNOWN",
        Vec::new(),
        Control::ExistingLoadError,
    )
    .await;
}

#[tokio::test]
async fn bbl_supplied_images_preserve_complete_stream_and_original_load_errors() {
    for definitions in ["32x24/PNG", "32x24/BTT_TFT"] {
        support::unchanged(
            project::BBL,
            definitions,
            vec![samples::asymmetric()],
            Control::CompleteStream,
        )
        .await;
    }
    support::unchanged(
        project::BBL,
        "bad, 1000x24/UNKNOWN",
        vec![samples::asymmetric()],
        Control::ExistingLoadError,
    )
    .await;
}
