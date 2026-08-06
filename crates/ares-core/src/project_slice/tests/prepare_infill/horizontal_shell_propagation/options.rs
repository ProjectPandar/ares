use super::ksr::digest::{event_sequence_digest, surface_sequence_digest};
use crate::{
    geometry::CoordinateScale,
    project_slice::{
        prepare_infill::horizontal_shell_propagation::{self, PropagationEvent},
        tests::support::KsrArchive,
    },
};

#[derive(Debug, PartialEq)]
struct Capture {
    top_layers: i64,
    top_thickness: u64,
    bottom_layers: i64,
    bottom_thickness: u64,
    density: u64,
    external_widths: i128,
    solid_widths: i128,
    scale: CoordinateScale,
    behavior: (i128, i128, usize),
}

fn moderate_archive() -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"ensure_moderate\"",
    );
    archive
}

fn capture(archive: KsrArchive) -> Capture {
    horizontal_shell_propagation::reset_hooks();
    let input = super::fixture::prepare_o25(archive.bytes());
    let traversal = &input.predecessor.objects[0];
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let (_, inputs) = prelude.object.as_parts();
    let record = inputs.iter().flatten().next().unwrap();
    let options = prelude.object.region_options(record);
    let typed = (
        i64::from(options.top_shell_layers.0),
        options.top_shell_thickness.0.to_bits(),
        i64::from(options.bottom_shell_layers.0),
        options.bottom_shell_thickness.0.to_bits(),
        options.sparse_infill_density.0.to_bits(),
        inputs.iter().flatten().fold(0_i128, |digest, input| {
            digest
                .wrapping_mul(1099511628211)
                .wrapping_add(i128::from(input.ext_perimeter_flow.width.to_bits()))
        }),
        inputs.iter().flatten().fold(0_i128, |digest, input| {
            digest
                .wrapping_mul(1099511628211)
                .wrapping_add(i128::from(input.solid_infill_flow.width.to_bits()))
        }),
        input.predecessor.scale,
    );
    let output = horizontal_shell_propagation::prepare(input).unwrap();
    let events = horizontal_shell_propagation::events();
    let behavior = (
        event_sequence_digest(&events),
        surface_sequence_digest(&output.objects),
        horizontal_shell_propagation::commits(),
    );
    horizontal_shell_propagation::dispose(output);
    Capture {
        top_layers: typed.0,
        top_thickness: typed.1,
        bottom_layers: typed.2,
        bottom_thickness: typed.3,
        density: typed.4,
        external_widths: typed.5,
        solid_widths: typed.6,
        scale: typed.7,
        behavior,
    }
}

#[test]
fn task22o26_model_part_ensure_mode_overrides_global_ensure_all() {
    let mut archive = KsrArchive::new();
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"ensure_vertical_shell_thickness\" value=\"ensure_moderate\"/>",
    );
    horizontal_shell_propagation::reset_hooks();
    let output = super::fixture::prepare(archive.bytes());
    let events = horizontal_shell_propagation::events();
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, PropagationEvent::EnsureAllSkip { .. }))
            .count(),
        0
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, PropagationEvent::Rebuild { .. }))
    );
    assert!(horizontal_shell_propagation::commits() > 0);
    horizontal_shell_propagation::dispose(output);
}

#[test]
fn task22o26_resolved_archive_options_drive_each_window_density_flow_and_scale() {
    let baseline = capture(moderate_archive());
    assert_eq!(
        (
            baseline.top_layers,
            baseline.top_thickness,
            baseline.bottom_layers,
            baseline.bottom_thickness,
            baseline.density,
            baseline.scale,
        ),
        (
            5,
            1.0_f64.to_bits(),
            3,
            0.0_f64.to_bits(),
            15.0_f64.to_bits(),
            CoordinateScale::Normal,
        )
    );

    let mut top_baseline = moderate_archive();
    top_baseline.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"1\"",
        "\"top_shell_thickness\": \"0\"",
    );
    let mut top = top_baseline.clone();
    top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_layers\": \"5\"",
        "\"top_shell_layers\": \"1\"",
    );
    let top_baseline = capture(top_baseline);
    let top = capture(top);
    assert_eq!(top.top_layers, 1);
    assert_ne!(top.behavior, top_baseline.behavior);

    let mut thin_top = moderate_archive();
    thin_top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_layers\": \"5\"",
        "\"top_shell_layers\": \"-1\"",
    );
    thin_top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"1\"",
        "\"top_shell_thickness\": \"0.2\"",
    );
    let mut thick_top = thin_top.clone();
    thick_top.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"0.2\"",
        "\"top_shell_thickness\": \"1.4\"",
    );
    let thin_top = capture(thin_top);
    let thick_top = capture(thick_top);
    assert_eq!(thick_top.top_thickness, 1.4_f64.to_bits());
    assert_ne!(thick_top.behavior, thin_top.behavior);

    let mut bottom = moderate_archive();
    bottom.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_layers\": \"3\"",
        "\"bottom_shell_layers\": \"1\"",
    );
    let bottom = capture(bottom);
    assert_eq!(bottom.bottom_layers, 1);
    assert_ne!(bottom.behavior, baseline.behavior);

    let mut bottom_thickness = moderate_archive();
    bottom_thickness.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_layers\": \"3\"",
        "\"bottom_shell_layers\": \"1\"",
    );
    bottom_thickness.replace_unique(
        "Metadata/project_settings.config",
        "\"bottom_shell_thickness\": \"0\"",
        "\"bottom_shell_thickness\": \"1\"",
    );
    let bottom_thickness = capture(bottom_thickness);
    assert_eq!(bottom_thickness.bottom_thickness, 1.0_f64.to_bits());
    assert_ne!(bottom_thickness.behavior, bottom.behavior);

    let mut density = moderate_archive();
    density.replace_unique(
        "Metadata/project_settings.config",
        "\"sparse_infill_density\": \"15%\"",
        "\"sparse_infill_density\": \"0%\"",
    );
    let density = capture(density);
    assert_eq!(density.density, 0.0_f64.to_bits());
    assert_ne!(density.behavior, baseline.behavior);

    let mut external_baseline = KsrArchive::new();
    external_baseline.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"none\"",
    );
    let mut external = external_baseline.clone();
    external.replace_unique(
        "Metadata/project_settings.config",
        "\"outer_wall_line_width\": \"0.42\"",
        "\"outer_wall_line_width\": \"0.8\"",
    );
    let external_baseline = capture(external_baseline);
    let external = capture(external);
    assert_ne!(external.external_widths, external_baseline.external_widths);
    assert_ne!(external.behavior.1, external_baseline.behavior.1);

    let mut solid = moderate_archive();
    solid.replace_unique(
        "Metadata/project_settings.config",
        "\"internal_solid_infill_line_width\": \"0.42\"",
        "\"internal_solid_infill_line_width\": \"0.8\"",
    );
    let solid = capture(solid);
    assert_ne!(solid.solid_widths, baseline.solid_widths);
    assert_ne!(solid.behavior.1, baseline.behavior.1);

    const NORMAL: &str = concat!(
        "\t\"printable_area\": [\r\n",
        "\t\t\"0x0\",\r\n",
        "\t\t\"256x0\",\r\n",
        "\t\t\"256x256\",\r\n",
        "\t\t\"0x256\"\r\n",
        "\t]",
    );
    const LARGE: &str = concat!(
        "\t\"printable_area\": [\r\n",
        "\t\t\"0x0\",\r\n",
        "\t\t\"2148x0\",\r\n",
        "\t\t\"2148x256\",\r\n",
        "\t\t\"0x256\"\r\n",
        "\t]",
    );
    let mut large = moderate_archive();
    large.replace_unique("Metadata/project_settings.config", NORMAL, LARGE);
    let large = capture(large);
    assert_eq!(large.scale, CoordinateScale::LargeBed);
    assert_ne!(large.behavior.1, baseline.behavior.1);
}

#[test]
fn task22o26_model_part_shell_window_override_reaches_resolved_behavior() {
    let mut baseline = moderate_archive();
    baseline.replace_unique(
        "Metadata/project_settings.config",
        "\"top_shell_thickness\": \"1\"",
        "\"top_shell_thickness\": \"0\"",
    );
    let mut archive = baseline.clone();
    let baseline = capture(baseline);
    archive.replace(
        "Metadata/model_settings.config",
        "    <part id=\"1\" subtype=\"normal_part\">",
        "    <part id=\"1\" subtype=\"normal_part\">\n      <metadata key=\"top_shell_layers\" value=\"1\"/>",
    );
    let changed = capture(archive);
    assert_eq!(changed.top_layers, 1);
    assert_ne!(changed.behavior, baseline.behavior);
}
