use crate::project_slice::{
    prepare_infill::{vertical_shell_assignment, vertical_shell_filtering},
    tests::support::KsrArchive,
};

#[test]
fn task22o24_inactive_typed_modes_are_complete_noops_without_geometry() {
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        let input = super::fixture::prepare_o23(archive.bytes());
        assert!(input.filters.iter().all(|object| {
            object
                .records
                .iter()
                .flatten()
                .all(|record| record.filtered_shell.is_empty())
        }));
        let before = snapshot(&input.objects);
        let pointers = input
            .objects
            .iter()
            .flat_map(|object| &object.records)
            .map(|record| record.as_ref().map(|record| record.fill_surfaces.as_ptr()))
            .collect::<Vec<_>>();
        vertical_shell_assignment::reset_geometry_hooks();
        let output = vertical_shell_assignment::prepare(input).unwrap();
        assert!(vertical_shell_assignment::geometry_events().is_empty());
        assert_eq!(snapshot(&output.objects), before);
        assert_eq!(
            output
                .objects
                .iter()
                .flat_map(|object| &object.records)
                .map(|record| record.as_ref().map(|record| record.fill_surfaces.as_ptr()))
                .collect::<Vec<_>>(),
            pointers
        );
        vertical_shell_assignment::dispose(output);
    }
    vertical_shell_filtering::reset_geometry_hooks();
}

fn snapshot(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut digest = 0x4f24_i128;
    for surface in objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
    {
        let (kind, expolygon, thickness, layers, angle, extra) = surface.as_parts();
        for value in [
            kind as i128,
            thickness.to_bits() as i128,
            layers as i128,
            angle.to_bits() as i128,
            extra as i128,
        ] {
            digest = digest.wrapping_mul(0x1000003d).wrapping_add(value);
        }
        for path in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
            for point in path.points() {
                digest = digest
                    .wrapping_mul(0x1000003d)
                    .wrapping_add(point.x() as i128)
                    .wrapping_add(point.y() as i128);
            }
        }
    }
    digest
}
