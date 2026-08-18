use std::fmt::Write as _;

use crate::geometry::{ExPolygon, Polygon};

use super::{OracleGroup, OracleLayer, OracleStage};

pub(super) fn encode_layer_metadata(output: &mut String, layer: &OracleLayer<'_>) {
    writeln!(
        output,
        "layer {} stage {} height_bits {} print_z_bits {} groups {}",
        layer.layer_id,
        match layer.stage {
            OracleStage::PostNarrow => "post-narrow",
        },
        layer.layer_height.to_bits(),
        layer.print_z.to_bits(),
        layer.groups.len()
    )
    .unwrap();
    writeln!(
        output,
        "lock_params skin_density {} skeleton_density {} skin_flow {} skeleton_flow {}",
        layer.lock_counts.skin_density,
        layer.lock_counts.skeleton_density,
        layer.lock_counts.skin_flow,
        layer.lock_counts.skeleton_flow
    )
    .unwrap();

    for (group_index, group) in layer.groups.iter().enumerate() {
        encode_group_metadata(output, group_index, group);
    }
    writeln!(output, "end_layer {}", layer.layer_id).unwrap();
}

fn encode_group_metadata(output: &mut String, group_index: usize, group: &OracleGroup<'_>) {
    let representative = group.representative;
    let params = group.params;
    writeln!(
        output,
        "group {group_index} region_id {} surface_type {} surface_thickness_bits {} surface_thickness_layers {} surface_bridge_angle_bits {} surface_extra_perimeters {}",
        group.region_id,
        representative.kind,
        representative.thickness.to_bits(),
        representative.thickness_layers,
        representative.bridge_angle.to_bits(),
        representative.extra_perimeters
    )
    .unwrap();
    writeln!(
        output,
        "params extruder {} pattern {} spacing_bits {} overlap_bits {} angle_bits {} fixed_angle {} bridge {} bridge_angle_bits {} density_bits {} multiline {} anchor_length_bits {} anchor_length_max_bits {}",
        params.extruder,
        params.pattern,
        params.spacing.to_bits(),
        params.overlap.to_bits(),
        params.angle.to_bits(),
        u8::from(params.fixed_angle),
        u8::from(params.bridge),
        params.bridge_angle.to_bits(),
        params.density.to_bits(),
        params.multiline,
        params.anchor_length.to_bits(),
        params.anchor_length_max.to_bits()
    )
    .unwrap();
    writeln!(
        output,
        "flow width_bits {} height_bits {} spacing_bits {} nozzle_bits {} bridge {} extrusion_role {} idx {} role_speed_bits {}",
        params.flow.width.to_bits(),
        params.flow.height.to_bits(),
        params.flow.spacing.to_bits(),
        params.flow.nozzle_diameter.to_bits(),
        u8::from(params.flow.bridge),
        params.extrusion_role,
        params.idx,
        params.role_speed.to_bits()
    )
    .unwrap();
    writeln!(
        output,
        "extras lateral_1_bits {} lateral_2_bits {} infill_lock_depth_bits {} skin_infill_depth_bits {} symmetric_y {} overhang_angle_bits {} gyroid_optimized {}",
        params.lateral_lattice_angle_1.to_bits(),
        params.lateral_lattice_angle_2.to_bits(),
        params.infill_lock_depth.to_bits(),
        params.skin_infill_depth.to_bits(),
        u8::from(params.symmetric_infill_y_axis),
        params.infill_overhang_angle.to_bits(),
        u8::from(params.gyroid_optimized)
    )
    .unwrap();
    write!(output, "region_id_group {}", group.region_id_group.len()).unwrap();
    for region_id in group.region_id_group {
        write!(output, " {region_id}").unwrap();
    }
    output.push('\n');

    // The C++ representative owns a moved-from ExPolygon. Its coordinates are
    // deliberately absent while the source serializer's section framing stays.
    output.push_str("surface expolygons 1\nend_surface\n");
    writeln!(output, "fills expolygons {}", group.fills.len()).unwrap();
    output.push_str("end_fills\n");
    writeln!(output, "no_overlap expolygons {}", group.no_overlap.len()).unwrap();
    output.push_str("end_no_overlap\n");
    writeln!(output, "end_group {group_index}").unwrap();
}

pub(super) fn encode_layer_geometry(records: &mut Vec<String>, layer: &OracleLayer<'_>) {
    for (group_index, group) in layer.groups.iter().enumerate() {
        encode_section_geometry(records, layer.layer_id, group_index, "fills", group.fills);
        encode_section_geometry(
            records,
            layer.layer_id,
            group_index,
            "no_overlap",
            group.no_overlap,
        );
    }
}

fn encode_section_geometry(
    records: &mut Vec<String>,
    layer_id: usize,
    group_index: usize,
    section: &str,
    expolygons: &[ExPolygon],
) {
    for expolygon in expolygons {
        let contour = ring_payload(expolygon.contour());
        records.push(format!("{layer_id}|{group_index}|{section}|C|{contour}"));
        for hole in expolygon.holes() {
            records.push(format!(
                "{layer_id}|{group_index}|{section}|H|{contour}|{}",
                ring_payload(hole)
            ));
        }
    }
}

fn ring_payload(polygon: &Polygon) -> String {
    let mut output = polygon.points().len().to_string();
    for point in polygon.points() {
        write!(output, " {},{}", point.x(), point.y()).unwrap();
    }
    output
}

pub(super) fn encode_layer_table(output: &mut String, layer: &OracleLayer<'_>) {
    let expolygons = layer
        .groups
        .iter()
        .map(|group| group.fills.len())
        .sum::<usize>();
    let holes = layer
        .groups
        .iter()
        .flat_map(|group| group.fills)
        .map(|expolygon| expolygon.holes().len())
        .sum::<usize>();
    let points = layer
        .groups
        .iter()
        .flat_map(|group| group.fills)
        .map(|expolygon| {
            expolygon.contour().points().len()
                + expolygon
                    .holes()
                    .iter()
                    .map(|hole| hole.points().len())
                    .sum::<usize>()
        })
        .sum::<usize>();
    writeln!(
        output,
        "{}\t{}\t{expolygons}\t{holes}\t{points}\t{}\t{}\t{}",
        layer.layer_id,
        layer.groups.len(),
        comma_values(&layer.groups, |group| group.representative.kind),
        comma_values(&layer.groups, |group| group.params.pattern),
        comma_values(&layer.groups, |group| group.params.extrusion_role)
    )
    .unwrap();
}

fn comma_values(groups: &[OracleGroup<'_>], value: impl Fn(&OracleGroup<'_>) -> u8) -> String {
    let mut output = String::new();
    for (index, group) in groups.iter().enumerate() {
        if index != 0 {
            output.push(',');
        }
        write!(output, "{}", value(group)).unwrap();
    }
    output
}
