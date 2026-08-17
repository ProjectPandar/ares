use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{
            horizontal_shell_promotion, horizontal_shell_propagation, surface_type_detection,
            vertical_shell_assignment, vertical_shell_filtering, vertical_shell_projection,
            vertical_shell_regularization, vertical_shell_trimming, vertical_shells,
        },
        slice_project,
        tests::support::{KsrArchive, metadata},
    },
};

#[tokio::test]
async fn task22o25_every_earlier_geometry_stage_precedes_promotion() {
    surface_type_detection::reset_geometry_hooks();
    surface_type_detection::fail_geometry_at(
        surface_type_detection::GeometryStep::TopSafetyDifference,
    );
    assert_precedes("surface-type detection geometry is outside the supported Clipper range").await;
    surface_type_detection::reset_geometry_hooks();

    vertical_shells::reset_geometry_hooks();
    vertical_shells::fail_geometry_at(vertical_shells::GeometryStep::Top);
    assert_precedes("vertical-shell cache geometry is outside the supported Clipper range").await;
    vertical_shells::reset_geometry_hooks();

    vertical_shell_projection::reset_geometry_hooks();
    vertical_shell_projection::fail_geometry_at(vertical_shell_projection::GeometryStep::TopVisit);
    assert_precedes("vertical-shell projection geometry is outside the supported Clipper range")
        .await;
    vertical_shell_projection::reset_geometry_hooks();

    vertical_shell_trimming::reset_geometry_hooks();
    vertical_shell_trimming::fail_geometry_at(vertical_shell_trimming::GeometryStep::SafetyOffset);
    assert_precedes(
        "vertical-shell internal trimming geometry is outside the supported Clipper range",
    )
    .await;
    vertical_shell_trimming::reset_geometry_hooks();

    vertical_shell_regularization::reset_geometry_hooks();
    vertical_shell_regularization::fail_geometry_at(
        vertical_shell_regularization::GeometryStep::Union,
    );
    assert_precedes(
        "vertical-shell regularization geometry is outside the supported Clipper range",
    )
    .await;
    vertical_shell_regularization::reset_geometry_hooks();

    vertical_shell_filtering::reset_geometry_hooks();
    vertical_shell_filtering::fail_geometry_at(
        vertical_shell_filtering::GeometryStep::NeighborIntersection,
    );
    assert_precedes(
        "vertical-shell tiny-island filtering geometry is outside the supported Clipper range",
    )
    .await;
    vertical_shell_filtering::reset_geometry_hooks();

    vertical_shell_assignment::reset_geometry_hooks();
    vertical_shell_assignment::fail_geometry_at(
        vertical_shell_assignment::GeometryStep::SolidIntersection,
    );
    assert_precedes(
        "vertical-shell fill-surface assignment geometry is outside the supported Clipper range",
    )
    .await;
    vertical_shell_assignment::reset_geometry_hooks();
}

#[tokio::test]
async fn task22o25_public_parse_error_disposes_o24_and_preserves_error() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"extra_solid_infills\": \"\"",
        "\"extra_solid_infills\": \"2147483648\"",
    );
    horizontal_shell_promotion::reset_hooks();
    horizontal_shell_propagation::reset_hooks();
    assert_eq!(
        slice_project(archive.bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput("invalid extra_solid_infills pattern".to_owned())
    );
    assert_eq!(horizontal_shell_promotion::invocations(), 1);
    assert_eq!(horizontal_shell_promotion::commits(), 0);
    assert_eq!(horizontal_shell_promotion::disposals(), 1);
    assert_eq!(horizontal_shell_propagation::invocations(), 0);
}

#[tokio::test]
async fn task22o25_every_earlier_capability_error_precedes_promotion() {
    for (from, to, feature) in [
        (
            "\"spiral_mode\": \"0\"",
            "\"spiral_mode\": \"1\"",
            "spiral_mode",
        ),
        (
            "\"counterbore_hole_bridging\": \"none\"",
            "\"counterbore_hole_bridging\": \"partiallybridge\"",
            "counterbore_hole_bridging",
        ),
        (
            "\"interface_shells\": \"0\"",
            "\"interface_shells\": \"1\"",
            "interface_shells",
        ),
        (
            "\"enable_extra_bridge_layer\": \"disabled\"",
            "\"enable_extra_bridge_layer\": \"apply_to_all\"",
            "enable_extra_bridge_layer",
        ),
    ] {
        let mut archive = KsrArchive::new();
        archive.replace_unique("Metadata/project_settings.config", from, to);
        horizontal_shell_promotion::reset_hooks();
        horizontal_shell_propagation::reset_hooks();
        assert_eq!(
            slice_project(archive.bytes(), metadata())
                .await
                .unwrap_err(),
            SliceError::UnsupportedProjectFeature(feature.to_owned())
        );
        assert_eq!(horizontal_shell_promotion::invocations(), 0);
        assert_eq!(horizontal_shell_promotion::disposals(), 0);
        assert_eq!(horizontal_shell_propagation::invocations(), 0);
    }

    let (multi_region, _) = crate::project_slice::tests::region_fixture::modifier_projects();
    horizontal_shell_promotion::reset_hooks();
    horizontal_shell_propagation::reset_hooks();
    assert_eq!(
        slice_project(multi_region, metadata()).await.unwrap_err(),
        SliceError::UnsupportedProjectFeature("multi_region_layer_slices".to_owned())
    );
    assert_eq!(horizontal_shell_promotion::invocations(), 0);
    assert_eq!(horizontal_shell_promotion::disposals(), 0);
    assert_eq!(horizontal_shell_propagation::invocations(), 0);
}

async fn assert_precedes(expected: &str) {
    horizontal_shell_promotion::reset_hooks();
    horizontal_shell_propagation::reset_hooks();
    assert_eq!(
        slice_project(KsrArchive::new().bytes(), metadata())
            .await
            .unwrap_err(),
        SliceError::InvalidInput(expected.to_owned())
    );
    assert_eq!(horizontal_shell_promotion::invocations(), 0);
    assert_eq!(horizontal_shell_promotion::disposals(), 0);
    assert_eq!(horizontal_shell_propagation::invocations(), 0);
}
