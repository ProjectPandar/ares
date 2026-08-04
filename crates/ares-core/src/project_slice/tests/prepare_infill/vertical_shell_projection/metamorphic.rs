use crate::project_slice::{
    prepare_infill::vertical_shell_projection::PreparedPostVerticalShellProjection,
    tests::support::KsrArchive,
};

use super::{fixture, ksr::projection_digest};

#[test]
fn task22o20_zip_repack_and_non_slicing_rename_preserve_projection() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline = projection_digest(&baseline.projections);
    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o20_renamed\"",
    );
    let renamed = fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(projection_digest(&renamed.projections), baseline);
}

#[test]
fn task22o20_component_x_scale_changes_source_and_projection_geometry() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = projection_digest(&baseline.projections);
    let baseline_span = geometry_spans(&baseline);
    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = fixture::prepare(scaled.bytes());
    assert_ne!(projection_digest(&scaled.projections), baseline_digest);
    let scaled_span = geometry_spans(&scaled);
    assert_eq!(scaled_span.0.0, baseline_span.0.0);
    assert_eq!(scaled_span.1.0, baseline_span.1.0);
    assert_ne!(scaled_span.0.1, baseline_span.0.1);
    assert_ne!(scaled_span.1.1, baseline_span.1.1);
    assert_eq!(visit_counts(&scaled), visit_counts(&baseline));
}

#[test]
fn task22o20_outer_width_changes_aligned_spacing_and_projection() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = projection_digest(&baseline.projections);
    let baseline_spacing = first_spacings(&baseline);
    let mut width = KsrArchive::new();
    width.replace_unique(
        "Metadata/project_settings.config",
        "\"outer_wall_line_width\": \"0.42\"",
        "\"outer_wall_line_width\": \"0.52\"",
    );
    let width = fixture::prepare(width.bytes());
    assert_ne!(first_spacings(&width), baseline_spacing);
    assert_ne!(projection_digest(&width.projections), baseline_digest);

    let mut part_width = KsrArchive::new();
    part_width.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"outer_wall_line_width\" value=\"0.52\"/>",
    );
    let part_width = fixture::prepare(part_width.bytes());
    assert_eq!(first_spacings(&part_width), first_spacings(&width));
    assert_eq!(
        projection_digest(&part_width.projections),
        projection_digest(&width.projections)
    );
}

fn first_spacings(prepared: &PreparedPostVerticalShellProjection) -> [i64; 2] {
    let records = &prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records;
    [
        records[0].as_ref().unwrap().external_spacing,
        records[1].as_ref().unwrap().external_spacing,
    ]
}

fn visit_counts(prepared: &PreparedPostVerticalShellProjection) -> (i32, i32, u64, u64) {
    let prelude = &prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let input = prelude.object.records[0].as_ref().unwrap();
    let options = prelude.object.region_options(input);
    (
        options.top_shell_layers.0,
        options.bottom_shell_layers.0,
        options.top_shell_thickness.0.to_bits(),
        options.bottom_shell_thickness.0.to_bits(),
    )
}

fn geometry_spans(prepared: &PreparedPostVerticalShellProjection) -> ((usize, i64), (usize, i64)) {
    let (source_index, source) = prepared.caches[0]
        .records
        .iter()
        .enumerate()
        .find_map(|(index, cache)| {
            cache
                .as_ref()
                .and_then(|cache| cache.top_surfaces.first())
                .map(|path| (index, path))
        })
        .expect("KSR must contain a populated O19 top path");
    let (projection_index, projection) = prepared.projections[0]
        .records
        .iter()
        .enumerate()
        .find_map(|(index, projection)| {
            projection
                .as_ref()
                .and_then(|projection| projection.shell.first())
                .map(|path| (index, path))
        })
        .expect("KSR must contain a populated O20 shell path");
    (
        (source_index, x_span(source.points())),
        (projection_index, x_span(projection.points())),
    )
}

fn x_span(points: &[crate::geometry::Point]) -> i64 {
    let (minimum, maximum) = points
        .iter()
        .map(|point| point.x())
        .fold((i64::MAX, i64::MIN), |(minimum, maximum), x| {
            (minimum.min(x), maximum.max(x))
        });
    maximum - minimum
}
