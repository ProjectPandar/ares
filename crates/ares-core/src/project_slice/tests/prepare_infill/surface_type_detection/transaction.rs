use std::sync::atomic::Ordering;

use crate::{
    SliceError,
    project_slice::{
        perimeters,
        prepare_infill::surface_type_detection::{
            self, GeometryStep, fail_geometry_at, geometry_events, reset_geometry_hooks,
        },
    },
};

use super::super::super::support::KsrArchive;

#[test]
fn task22o17_geometry_failure_consumes_the_unmoved_predecessor_transactionally() {
    let prepared =
        perimeters::prepare_post_layer_region_perimeters(&KsrArchive::new().bytes()).unwrap();
    let (weak, dropped) = prepared.predecessor.drop_probe_observer();
    reset_geometry_hooks();
    fail_geometry_at(GeometryStep::TopSafetyDifference);
    let error = match surface_type_detection::prepare(prepared) {
        Err(error) => error,
        Ok(_) => panic!("injected O17 geometry failure must not produce a successor"),
    };
    assert_eq!(
        error,
        SliceError::InvalidInput(
            "surface-type detection geometry is outside the supported Clipper range".to_owned()
        )
    );
    assert!(geometry_events().contains(&GeometryStep::TopSafetyDifference));
    assert!(weak.upgrade().is_none());
    assert!(dropped.load(Ordering::SeqCst));
    reset_geometry_hooks();
}

#[test]
fn task22o17_interface_error_has_key_major_precedence() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"interface_shells\": \"0\"",
        "\"interface_shells\": \"1\"",
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"enable_extra_bridge_layer\": \"disabled\"",
        "\"enable_extra_bridge_layer\": \"apply_to_all\"",
    );
    let prepared = perimeters::prepare_post_layer_region_perimeters(&archive.bytes()).unwrap();
    reset_geometry_hooks();
    let error = match surface_type_detection::prepare(prepared) {
        Err(error) => error,
        Ok(_) => panic!("invalid O17 preflight must not produce a successor"),
    };
    assert_eq!(
        error,
        SliceError::UnsupportedProjectFeature("interface_shells".to_owned())
    );
    assert!(geometry_events().is_empty());
}
