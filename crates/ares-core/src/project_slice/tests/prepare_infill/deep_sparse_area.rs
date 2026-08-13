use std::fmt::Write;

use sha2::{Digest, Sha256};

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

const EXPECTED: [(usize, usize, usize, &str); 18] = [
    (
        15,
        8,
        547,
        "3eeafecdceb6303eee94b921572e5abdb577f0b4a0642d5be2d39f046d05bda1",
    ),
    (
        30,
        9,
        520,
        "d1bdba743a99c62b3d37c6ecd3e9fc7634e4142c3d2cc73993bf7b8f3bb8721f",
    ),
    (
        31,
        9,
        600,
        "80c106d41b235d3eafa358f3e919595086bb72d154479f1c8d4c10aec23d4fcc",
    ),
    (
        32,
        10,
        576,
        "35b8cfaf23915db855e3203ef693fed71324d51e0e3c2f4ca5e411fc06529579",
    ),
    (
        41,
        9,
        471,
        "95704aedd5237346150328527f04ccd69c4275accef745f403393ff8a6872757",
    ),
    (
        45,
        15,
        1_256,
        "b731cb5604d6b7e3457148a526c84633a9c897af760e667378e0b0de20333f6f",
    ),
    (
        60,
        9,
        313,
        "fcbc0a6ac76ba231ce5a536304036e14e20cda2a8f1486ccb4d42a7d39b16064",
    ),
    (
        65,
        9,
        295,
        "3766ea336da4ad3a20c4f9759421c5da5d0178700bd3530e6645ea25088aa696",
    ),
    (
        70,
        9,
        283,
        "8731e8f5a2f7bc7ad5ff36673d12ab73950c69a08208a78df7183f6cc7d96d82",
    ),
    (
        75,
        4,
        106,
        "0e32fa3eefc68a813b5794f16f03d3a10b56138fba2801967ca224e1801a0118",
    ),
    (
        82,
        4,
        105,
        "0d256788bd4496e527f886c4ed5070bfbd8a2570e171255ac438ab4f35b09403",
    ),
    (
        85,
        4,
        119,
        "0f8c16c9d7889db21736b85e540eb1465d744d08846ff7623819818c1f9d2602",
    ),
    (
        90,
        4,
        122,
        "2635a9c7a8096e40214bf03c5f58367d535f5fdd45c571658215fca27630c0d8",
    ),
    (
        105,
        3,
        98,
        "3563b5ff317f9f88f4710213ed690d80a47094266265510918b66951385dd326",
    ),
    (
        116,
        3,
        89,
        "2ddb0c98d9852e511832449d9724c3d7eb2e3e6cd4a09ff6d2005253e453131c",
    ),
    (
        125,
        3,
        104,
        "58cf0ba6c54e854ba174b56caafdb28b1e134235649b2c9329cc29017c6813f0",
    ),
    (
        136,
        2,
        26,
        "78ebbcdafe4ebc26d3ff0210d713dcd3179265b609a752f90cb5f197eb068ab5",
    ),
    (
        255,
        1,
        11,
        "144a3e83b0e507fe63c1e958c045bcbfca1bf8a5a50610d636d65521e07d745a",
    ),
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
    let mut aggregate = Vec::new();
    let mut polygon_total = 0;
    let mut point_total = 0;
    for ((layer, polygons, points, digest), bytes) in EXPECTED.iter().zip(&first) {
        assert_eq!(decode_u64(bytes, 0) as usize, *layer);
        assert_eq!(decode_u64(bytes, 8) as usize, *polygons, "layer {layer}");
        assert_eq!(point_count(bytes), *points, "layer {layer}");
        assert_eq!(sha256(bytes), *digest, "layer {layer}");
        polygon_total += *polygons;
        point_total += *points;
        aggregate.extend_from_slice(bytes);
    }
    assert_eq!((polygon_total, point_total), (115, 5_641));
    assert_eq!(aggregate.len(), 91_464);
    assert_eq!(
        sha256(&aggregate),
        "f28db7dd3fc63155752ba5c33d4cd6338b2e311d83eb973c473d7f65268aa92a"
    );
}

fn run(
    layers: &[DeepSparseLayer<'_>],
    target_flow_height: f32,
    scale: crate::geometry::CoordinateScale,
) -> Vec<Vec<u8>> {
    EXPECTED
        .iter()
        .map(|&(layer, _, _, _)| {
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

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
}
