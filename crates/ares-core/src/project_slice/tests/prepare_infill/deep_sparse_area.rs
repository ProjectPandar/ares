use crate::{
    FloatOrPercent, Percent,
    geometry::Polygon,
    project_slice::{
        perimeters::flow::resolve_thick_solid_infill_bridge_flow,
        prepare_infill::{
            bridge_over_infill::deep_sparse_area::{
                DeepSparseLayer, gather_deep_sparse_infill_area,
            },
            external_surfaces,
        },
        tests::support::KsrArchive,
    },
};

use super::horizontal_shell_propagation;

const TARGET_LAYERS: [usize; 18] = [
    15, 30, 31, 32, 41, 45, 60, 65, 70, 75, 82, 85, 90, 105, 116, 125, 136, 255,
];

#[test]
fn task22o47_real_ksr_deep_sparse_areas_are_repeatable_and_preserve_input() {
    let horizontal = horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let external = external_surfaces::prepare(horizontal).unwrap();
    let horizontal = &external.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (compensated, inputs) = prelude.object.as_parts();
    let (post_regions, _) = compensated.as_parts();
    let (plan, _, _) = post_regions.as_parts();
    let first_input = inputs.iter().flatten().next().unwrap();
    let region = prelude.object.region_options(first_input);
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    assert_eq!(
        region.bridge_line_width,
        FloatOrPercent::Percent(Percent(100.0))
    );
    assert_eq!(region.bridge_flow.0, 1.0);
    let bridge_flow = resolve_thick_solid_infill_bridge_flow(region, nozzles).unwrap();
    assert_eq!(
        (
            bridge_flow.width.to_bits(),
            bridge_flow.height.to_bits(),
            bridge_flow.spacing.to_bits(),
            bridge_flow.nozzle_diameter.to_bits(),
            bridge_flow.bridge,
            bridge_flow.mm3_per_mm.to_bits(),
        ),
        (
            0x3ecc_cccd,
            0x3ecc_cccd,
            0x3ee6_6667,
            0x3ecc_cccd,
            true,
            0x3fc0_15bf_a000_0000,
        )
    );
    let target_flow_height = bridge_flow.height * 0.9_f32;
    let region_before = region.clone();
    let nozzles_before = nozzles.clone();
    let scale_before = traversal.scale;
    let layers = plan
        .layers
        .iter()
        .enumerate()
        .map(|(index, planned)| DeepSparseLayer {
            planned,
            fill_surfaces: horizontal.objects[0].records[index]
                .as_ref()
                .map_or(&[], |record| record.fill_surfaces.as_slice()),
            sparse_infill_density_percent: region.sparse_infill_density.0,
        })
        .collect::<Vec<_>>();
    let before = snapshot(&layers);

    let first = run(&layers, target_flow_height, traversal.scale);
    let second = run(&layers, target_flow_height, traversal.scale);

    assert_eq!(first, second);
    assert_eq!(snapshot(&layers), before);
    assert_eq!(region, &region_before);
    assert_eq!(
        traversal.resolved.views.full.project.print.nozzle_diameter,
        nozzles_before
    );
    assert_eq!(traversal.scale, scale_before);
    for (&layer, bytes) in TARGET_LAYERS.iter().zip(&first) {
        assert_eq!(decode_u64(bytes, 0) as usize, layer);
        let polygons = decode_u64(bytes, 8) as usize;
        assert!(polygons > 0, "layer {layer}");
        assert!(point_count(bytes) >= polygons * 3, "layer {layer}");
    }
}

fn run(
    layers: &[DeepSparseLayer<'_>],
    target_flow_height: f32,
    scale: crate::geometry::CoordinateScale,
) -> Vec<Vec<u8>> {
    TARGET_LAYERS
        .iter()
        .map(|&layer| {
            let output =
                gather_deep_sparse_infill_area(layers, layer, target_flow_height, scale).unwrap();
            serialize(layer, &output)
        })
        .collect()
}

fn serialize(layer: usize, polygons: &[Polygon]) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(&(layer as u64).to_le_bytes());
    output.extend_from_slice(&(polygons.len() as u64).to_le_bytes());
    for polygon in polygons {
        append_polygon(&mut output, polygon);
    }
    output
}

fn append_polygon(output: &mut Vec<u8>, polygon: &Polygon) {
    output.extend_from_slice(&(polygon.points().len() as u64).to_le_bytes());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

fn point_count(bytes: &[u8]) -> usize {
    let mut offset = 16;
    let mut points = 0;
    for _ in 0..decode_u64(bytes, 8) {
        let count = decode_u64(bytes, offset) as usize;
        offset += 8 + count * 16;
        points += count;
    }
    assert_eq!(offset, bytes.len());
    points
}

fn decode_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
}

fn snapshot(layers: &[DeepSparseLayer<'_>]) -> Vec<u8> {
    let mut output = Vec::new();
    for layer in layers {
        output.extend_from_slice(&layer.planned.id.to_le_bytes());
        output.extend_from_slice(&layer.planned.height.to_bits().to_le_bytes());
        output.extend_from_slice(&layer.planned.print_z.to_bits().to_le_bytes());
        output.extend_from_slice(&layer.planned.slice_z.to_bits().to_le_bytes());
        output.extend_from_slice(&layer.sparse_infill_density_percent.to_bits().to_le_bytes());
        for surface in layer.fill_surfaces {
            let (kind, expolygon, thickness, count, angle, extra) = surface.as_parts();
            output.extend_from_slice(
                format!(
                    "{kind:?}:{}:{count}:{}:{extra}|",
                    thickness.to_bits(),
                    angle.to_bits()
                )
                .as_bytes(),
            );
            append_polygon(&mut output, expolygon.contour());
            for hole in expolygon.holes() {
                append_polygon(&mut output, hole);
            }
        }
    }
    output
}
