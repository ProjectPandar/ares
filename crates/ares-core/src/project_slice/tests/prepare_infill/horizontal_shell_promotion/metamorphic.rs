use crate::project_slice::{
    prepare_infill::horizontal_shell_promotion, tests::support::KsrArchive,
};

#[test]
fn task22o25_zip_order_compression_and_timestamp_do_not_change_evidence() {
    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o25_renamed\"",
    );
    let captures = [
        KsrArchive::new().bytes(),
        KsrArchive::new().bytes_stored_reverse(),
        KsrArchive::new().bytes_with_timestamp(),
        renamed.bytes_stored_reverse(),
    ]
    .map(capture);
    assert_eq!(captures[1], captures[0]);
    assert_eq!(captures[2], captures[0]);
    assert_eq!(captures[3], captures[0]);
}

#[test]
fn task22o25_transform_and_printable_area_change_only_predecessor_geometry() {
    let baseline = capture(KsrArchive::new().bytes());

    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = capture(scaled.bytes());
    assert_eq!((scaled.0, scaled.1), (baseline.0, baseline.1));
    assert_ne!(scaled.2, baseline.2);

    const NORMAL_AREA: &str = concat!(
        "\t\"printable_area\": [\r\n",
        "\t\t\"0x0\",\r\n",
        "\t\t\"256x0\",\r\n",
        "\t\t\"256x256\",\r\n",
        "\t\t\"0x256\"\r\n",
        "\t]",
    );
    const LARGE_AREA: &str = concat!(
        "\t\"printable_area\": [\r\n",
        "\t\t\"0x0\",\r\n",
        "\t\t\"2148x0\",\r\n",
        "\t\t\"2148x256\",\r\n",
        "\t\t\"0x256\"\r\n",
        "\t]",
    );
    let mut large = KsrArchive::new();
    large.replace_unique("Metadata/project_settings.config", NORMAL_AREA, LARGE_AREA);
    let large = capture(large.bytes());
    assert_eq!((large.0, large.1), (baseline.0, baseline.1));
    assert_ne!(large.2, baseline.2);
}

fn capture(bytes: Vec<u8>) -> (usize, usize, i128) {
    horizontal_shell_promotion::reset_hooks();
    let input = super::fixture::prepare_o24(bytes);
    let before = digest(&input.objects);
    let output = horizontal_shell_promotion::prepare(input).unwrap();
    let commits = horizontal_shell_promotion::commits();
    let events = horizontal_shell_promotion::events().len();
    let after = digest(&output.objects);
    assert_eq!(after, before);
    horizontal_shell_promotion::dispose(output);
    (commits, events, after)
}

fn digest(
    objects: &[crate::project_slice::prepare_infill::surface_type_detection::PreparedSurfaceTypeObject],
) -> i128 {
    let mut digest = 0x4f25_i128;
    for surface in objects
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.fill_surfaces)
    {
        digest = digest
            .wrapping_mul(0x1000003d)
            .wrapping_add(surface.as_parts().0 as i128);
        for path in
            std::iter::once(surface.as_parts().1.contour()).chain(surface.as_parts().1.holes())
        {
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
