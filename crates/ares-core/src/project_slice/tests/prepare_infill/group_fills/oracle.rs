use std::fmt::Write;

use crate::{
    geometry::{ExPolygon, Polygon},
    project_slice::group_fills::{GroupedFills, SurfaceFillPattern},
};

#[derive(Clone, Copy)]
pub(super) struct LayerHeader {
    pub(super) id: usize,
    pub(super) height: f64,
    pub(super) print_z: f64,
}

pub(super) fn metadata(header: LayerHeader, grouped: &GroupedFills) -> Vec<u8> {
    metadata_at_stage(header, grouped, "pre-narrow")
}

fn metadata_at_stage(header: LayerHeader, grouped: &GroupedFills, stage: &str) -> Vec<u8> {
    let mut output = String::new();
    writeln!(
        output,
        "layer {} stage {stage} height_bits {} print_z_bits {} groups {}",
        header.id,
        header.height.to_bits(),
        header.print_z.to_bits(),
        grouped.surface_fills.len()
    )
    .unwrap();
    writeln!(
        output,
        "lock_params skin_density {} skeleton_density {} skin_flow {} skeleton_flow {}",
        grouped.lock_region_param.skin_density_params.len(),
        grouped.lock_region_param.skeleton_density_params.len(),
        grouped.lock_region_param.skin_flow_params.len(),
        grouped.lock_region_param.skeleton_flow_params.len()
    )
    .unwrap();

    for (index, fill) in grouped.surface_fills.iter().enumerate() {
        let representative = &fill.representative;
        let params = &fill.params;
        writeln!(
            output,
            "group {index} region_id {} surface_type {} surface_thickness_bits {} surface_thickness_layers {} surface_bridge_angle_bits {} surface_extra_perimeters {}",
            fill.region_id,
            representative.kind as u8,
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
            layer_255_pattern_ordinal(params.pattern),
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
            super::portable_oracle::extrusion_role_rank(params.extrusion_role),
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
        write!(output, "region_id_group {}", fill.region_id_group.len()).unwrap();
        for region_id in &fill.region_id_group {
            write!(output, " {region_id}").unwrap();
        }
        output.push('\n');
        output.push_str("surface expolygons 1\n");
        output.push_str("end_surface\n");
        writeln!(output, "fills expolygons {}", fill.expolygons.len()).unwrap();
        output.push_str("end_fills\n");
        writeln!(
            output,
            "no_overlap expolygons {}",
            fill.no_overlap_expolygons.len()
        )
        .unwrap();
        output.push_str("end_no_overlap\n");
        writeln!(output, "end_group {index}").unwrap();
    }
    writeln!(output, "end_layer {}", header.id).unwrap();
    output.into_bytes()
}

pub(super) fn authoritative_geometry(grouped: &GroupedFills) -> Vec<u8> {
    let mut output = String::new();
    for fill in &grouped.surface_fills {
        write_expolygons(&mut output, "fills", &fill.expolygons);
        write_expolygons(&mut output, "no_overlap", &fill.no_overlap_expolygons);
    }
    output.into_bytes()
}

fn layer_255_pattern_ordinal(pattern: SurfaceFillPattern) -> u8 {
    match pattern {
        SurfaceFillPattern::Configured(pattern) => {
            super::portable_oracle::configured_pattern_rank(pattern)
        }
        SurfaceFillPattern::ConcentricInternal => 29,
    }
}

fn write_expolygons(output: &mut String, name: &str, expolygons: &[ExPolygon]) {
    writeln!(output, "{name} expolygons {}", expolygons.len()).unwrap();
    for (index, expolygon) in expolygons.iter().enumerate() {
        writeln!(
            output,
            "expolygon {index} holes {}",
            expolygon.holes().len()
        )
        .unwrap();
        write_polygon(output, "contour", 0, expolygon.contour());
        for (hole_index, hole) in expolygon.holes().iter().enumerate() {
            write_polygon(output, "hole", hole_index, hole);
        }
        writeln!(output, "end_expolygon {index}").unwrap();
    }
    writeln!(output, "end_{name}").unwrap();
}

fn write_polygon(output: &mut String, kind: &str, index: usize, polygon: &Polygon) {
    write!(output, "{kind} {index} points {}", polygon.points().len()).unwrap();
    for point in polygon.points() {
        write!(output, " {},{}", point.x(), point.y()).unwrap();
    }
    output.push('\n');
}
