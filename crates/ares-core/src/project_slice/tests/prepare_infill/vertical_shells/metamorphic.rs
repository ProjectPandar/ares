use crate::{
    geometry::Polygon,
    project_slice::{
        region_slices::RegionSurfaceKind,
        tests::{prepare_infill::fill_surfaces::ksr::unrelated_checksum, support::KsrArchive},
    },
};

use super::{fixture, ksr::cache_digest};

#[test]
fn task22o19_zip_repack_and_non_slicing_names_preserve_cache() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline = cache_digest(&baseline.caches);
    let mut renamed = KsrArchive::new();
    renamed.replace(
        "Metadata/model_settings.config",
        "value=\"ksr_fdmtest_v4.drc\"",
        "value=\"task22o19_renamed\"",
    );
    let renamed = fixture::prepare(renamed.bytes_stored_reverse());
    assert_eq!(cache_digest(&renamed.caches), baseline);
}

#[test]
fn task22o19_flow_width_and_component_scale_change_cache_geometry() {
    let baseline = fixture::prepare(KsrArchive::new().bytes());
    let baseline_digest = cache_digest(&baseline.caches);
    let baseline_span = bottom_span_relation(&baseline);

    let mut width = KsrArchive::new();
    width.replace_unique(
        "Metadata/project_settings.config",
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"0.52\"",
    );
    let width = prepare_with_unrelated_guard(width.bytes());
    let width_digest = cache_digest(&width.caches);
    assert_eq!(first_spacings(&width), [457_079, 477_079]);
    assert_ne!(width_digest, baseline_digest);

    let mut fallback = KsrArchive::new();
    fallback.replace_unique(
        "Metadata/project_settings.config",
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"0\"",
    );
    fallback.replace_unique(
        "Metadata/project_settings.config",
        "\"line_width\": \"0.42\"",
        "\"line_width\": \"0.52\"",
    );
    let fallback = prepare_with_unrelated_guard(fallback.bytes());
    assert_eq!(cache_digest(&fallback.caches), width_digest);
    assert_eq!(first_spacings(&fallback), [457_079, 477_079]);

    let mut part_width = KsrArchive::new();
    part_width.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"internal_solid_infill_line_width\" value=\"0.52\"/>",
    );
    let part_width = prepare_with_unrelated_guard(part_width.bytes());
    assert_eq!(cache_digest(&part_width.caches), width_digest);
    assert_eq!(first_spacings(&part_width), [457_079, 477_079]);

    let mut scaled = KsrArchive::new();
    scaled.replace_unique(
        "3D/3dmodel.model",
        "transform=\"1 0 0 0 1 0 0 0 1 0 0 0\"",
        "transform=\"2 0 0 0 1 0 0 0 1 0 0 0\"",
    );
    let scaled = fixture::prepare(scaled.bytes());
    assert_ne!(cache_digest(&scaled.caches), baseline_digest);
    let scaled_span = bottom_span_relation(&scaled);
    assert_eq!(scaled_span.2, baseline_span.2);
    assert_ne!(scaled_span.0, baseline_span.0);
    assert_ne!(scaled_span.1, baseline_span.1);
}

fn prepare_with_unrelated_guard(
    bytes: impl AsRef<[u8]>,
) -> crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache {
    let prepared = fixture::prepare_o18(bytes);
    let before = unrelated_checksum(&prepared.predecessor, &prepared.objects);
    let output = crate::project_slice::prepare_infill::vertical_shells::prepare(prepared).unwrap();
    assert_eq!(
        unrelated_checksum(&output.predecessor, &output.objects),
        before
    );
    output
}

fn first_spacings(
    prepared: &crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
) -> [i64; 2] {
    let records = &prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records;
    [
        records[0].as_ref().unwrap().solid_infill_spacing,
        records[1].as_ref().unwrap().solid_infill_spacing,
    ]
}

fn bottom_span_relation(
    prepared: &crate::project_slice::prepare_infill::vertical_shells::PreparedPostVerticalShellCache,
) -> (i64, i64, i64) {
    let record = prepared.objects[0].records[0].as_ref().unwrap();
    let source = record
        .slices
        .iter()
        .find(|surface| {
            matches!(
                surface.as_parts().0,
                RegionSurfaceKind::Bottom | RegionSurfaceKind::BottomBridge
            )
        })
        .unwrap()
        .as_parts()
        .1
        .contour();
    let cache = &prepared.caches[0].records[0]
        .as_ref()
        .unwrap()
        .bottom_surfaces[0];
    let prelude = &prepared.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let spacing = prelude.records[0].as_ref().unwrap().solid_infill_spacing;
    let expansion = ((spacing as f32) * 0.05_f32).round() as i64;
    let source_span = x_span(source);
    let cache_span = x_span(cache);
    assert_eq!(cache_span - source_span, 2 * expansion);
    (source_span, cache_span, expansion)
}

fn x_span(path: &Polygon) -> i64 {
    let (minimum, maximum) = path
        .points()
        .iter()
        .map(|point| point.x())
        .fold((i64::MAX, i64::MIN), |(minimum, maximum), x| {
            (minimum.min(x), maximum.max(x))
        });
    maximum - minimum
}
