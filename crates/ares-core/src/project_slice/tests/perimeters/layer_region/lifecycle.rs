use crate::{project_slice::perimeters::layer_region, slice_project};

use super::super::super::support::{KsrArchive, metadata};

const CONFIG: &str = "Metadata/project_settings.config";

#[tokio::test]
async fn arachne_filled_counterbore_reaches_layer_region_lifecycle() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        CONFIG,
        "\"counterbore_hole_bridging\": \"none\"",
        "\"counterbore_hole_bridging\": \"sacrificiallayer\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"wall_generator\": \"classic\"",
        "\"wall_generator\": \"arachne\"",
    );
    layer_region::reset_finish_invocations();
    let output = slice_project(archive.bytes(), metadata(), crate::SlicerContext::new(0))
        .await
        .unwrap();
    assert!(!output.is_empty());
    assert_eq!(layer_region::finish_invocations(), 1);
}
