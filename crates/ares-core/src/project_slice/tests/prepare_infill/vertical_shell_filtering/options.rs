use crate::{
    geometry::CoordinateScale,
    project_slice::{prepare_infill::vertical_shell_filtering, tests::support::KsrArchive},
};

#[test]
fn task22o23_inactive_typed_modes_produce_empty_filters_without_geometry() {
    for mode in ["none", "ensure_critical_only", "ensure_moderate"] {
        let mut archive = KsrArchive::new();
        archive.replace_unique(
            "Metadata/project_settings.config",
            "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
            &format!("\"ensure_vertical_shell_thickness\": \"{mode}\""),
        );
        vertical_shell_filtering::reset_geometry_hooks();
        let output = super::fixture::prepare(archive.bytes());
        assert!(output.filters.iter().all(|object| {
            object
                .records
                .iter()
                .flatten()
                .all(|record| record.filtered_shell.is_empty())
        }));
        assert!(vertical_shell_filtering::geometry_events().is_empty());
        vertical_shell_filtering::dispose(output);
    }
}

#[test]
fn task22o23_model_part_ensure_precedence_reaches_filtering() {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"none\"",
    );
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"ensure_vertical_shell_thickness\" value=\"ensure_all\"/>",
    );
    vertical_shell_filtering::reset_geometry_hooks();
    let output = super::fixture::prepare(archive.bytes());
    assert!(!vertical_shell_filtering::geometry_events().is_empty());
    vertical_shell_filtering::dispose(output);
}

#[test]
fn task22o23_typed_solid_infill_width_changes_retained_spacing_and_thresholds() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let baseline_spacings = spacings(&baseline);
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"0.55\"",
    );
    let changed = super::fixture::prepare(archive.bytes());
    let changed_spacings = spacings(&changed);
    let changed_index = baseline_spacings
        .iter()
        .zip(&changed_spacings)
        .position(|(before, after)| before != after)
        .unwrap();
    let baseline_bits = vertical_shell_filtering::threshold_bits(
        baseline_spacings[changed_index],
        baseline.predecessor.scale,
    );
    let changed_bits = vertical_shell_filtering::threshold_bits(
        changed_spacings[changed_index],
        changed.predecessor.scale,
    );
    assert_ne!(changed_bits[0], baseline_bits[0]);
    assert_ne!(changed_bits[5], baseline_bits[5]);
    assert_ne!(changed_bits[6], baseline_bits[6]);
    assert_ne!(survivor_signature(&changed), survivor_signature(&baseline));
    vertical_shell_filtering::dispose(baseline);
    vertical_shell_filtering::dispose(changed);
}

#[test]
fn task22o23_model_part_spacing_precedence_changes_survivors() {
    let baseline = super::fixture::prepare(KsrArchive::new().bytes());
    let mut archive = KsrArchive::new();
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"internal_solid_infill_line_width\" value=\"0.8\"/>",
    );
    let changed = super::fixture::prepare(archive.bytes());
    assert_ne!(spacings(&changed), spacings(&baseline));
    assert_ne!(survivor_signature(&changed), survivor_signature(&baseline));
    vertical_shell_filtering::dispose(baseline);
    vertical_shell_filtering::dispose(changed);
}

#[test]
fn task22o23_printable_area_selects_scaled_constants_and_epsilon() {
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
    let normal = super::fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(normal.predecessor.scale, CoordinateScale::Normal);
    let spacing = normal.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .iter()
        .flatten()
        .next()
        .unwrap()
        .solid_infill_spacing;
    let normal_bits = vertical_shell_filtering::threshold_bits(spacing, normal.predecessor.scale);
    assert_eq!(normal_bits[1], 1_500_000);
    assert_eq!(normal_bits[3], 8_000_000);
    assert_eq!(
        vertical_shell_filtering::epsilon_bits(normal.predecessor.scale),
        0x42c8_0000
    );

    let mut archive = KsrArchive::new();
    archive.replace_unique("Metadata/project_settings.config", NORMAL_AREA, LARGE_AREA);
    let large = super::fixture::prepare(archive.bytes());
    assert_eq!(large.predecessor.scale, CoordinateScale::LargeBed);
    let large_spacing = large.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .iter()
        .flatten()
        .next()
        .unwrap()
        .solid_infill_spacing;
    let large_bits =
        vertical_shell_filtering::threshold_bits(large_spacing, large.predecessor.scale);
    assert_eq!(large_bits[1], 150_000);
    assert_eq!(large_bits[3], 799_999);
    assert_eq!(
        vertical_shell_filtering::epsilon_bits(large.predecessor.scale),
        0x4120_0000
    );
    assert_eq!(survivor_count(&normal), survivor_count(&large));
    assert_physically_corresponding(&normal, &large);
    vertical_shell_filtering::dispose(normal);
    vertical_shell_filtering::dispose(large);
}

fn spacings(output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering) -> Vec<i64> {
    output.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .records
        .iter()
        .flatten()
        .map(|record| record.solid_infill_spacing)
        .collect()
}

fn survivor_count(output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering) -> usize {
    output
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .map(|record| record.filtered_shell.len())
        .sum()
}

fn survivor_signature(
    output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) -> (usize, i128) {
    let mut count = 0;
    let mut digest = 0x5355_5256_i128;
    for expolygon in output
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.filtered_shell)
    {
        count += 1;
        for path in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
            digest = digest.wrapping_mul(0x100_0000_01b3);
            digest = digest.wrapping_add(path.points().len() as i128);
            for point in path.points() {
                digest = digest.wrapping_mul(0x100_0000_01b3);
                digest = digest.wrapping_add(point.x() as i128);
                digest = digest.wrapping_mul(0x100_0000_01b3);
                digest = digest.wrapping_add(point.y() as i128);
            }
        }
    }
    (count, digest)
}

fn assert_physically_corresponding(
    normal: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
    large: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) {
    let normal_bounds = survivor_bounds(normal);
    let large_bounds = survivor_bounds(large);
    let coordinate_tolerance = 8.0 * large.predecessor.scale.factor();
    for (normal_value, large_value) in normal_bounds.into_iter().zip(large_bounds) {
        assert!((normal_value - large_value).abs() <= coordinate_tolerance);
    }
    let normal_area = survivor_physical_area(normal);
    let large_area = survivor_physical_area(large);
    let area_tolerance = normal_area.abs().max(large_area.abs()) * 1.0e-4 + 1.0e-5;
    assert!((normal_area - large_area).abs() <= area_tolerance);
}

fn survivor_bounds(
    output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) -> [f64; 4] {
    let mut points = output
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.filtered_shell)
        .flat_map(|expolygon| std::iter::once(expolygon.contour()).chain(expolygon.holes()))
        .flat_map(|path| path.points());
    let first = points.next().unwrap();
    let scale = output.predecessor.scale;
    let first_x = scale.unscale(first.x());
    let first_y = scale.unscale(first.y());
    points.fold([first_x, first_x, first_y, first_y], |bounds, point| {
        let x = scale.unscale(point.x());
        let y = scale.unscale(point.y());
        [
            bounds[0].min(x),
            bounds[1].max(x),
            bounds[2].min(y),
            bounds[3].max(y),
        ]
    })
}

fn survivor_physical_area(
    output: &vertical_shell_filtering::PreparedPostVerticalShellFiltering,
) -> f64 {
    let scale = output.predecessor.scale.factor();
    output
        .filters
        .iter()
        .flat_map(|object| &object.records)
        .flatten()
        .flat_map(|record| &record.filtered_shell)
        .map(|expolygon| expolygon.area() * scale.powi(2))
        .sum()
}
