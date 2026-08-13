use std::fmt::Write;

use sha2::{Digest, Sha256};

use crate::{
    geometry::Polyline,
    project_slice::{
        prepare_infill::{
            bridge_over_infill::sparse_anchoring::{
                SparseAnchoringLayer, generate_sparse_infill_polylines_for_anchoring,
            },
            external_surfaces,
        },
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};

const EXPECTED: [(usize, usize, usize, &str); 18] = [
    (
        14,
        17,
        746,
        "a1e692258f4b5002e0e4469d1d256d6c0b41b81b79a4b06a3b4c4446daa118b0",
    ),
    (
        29,
        19,
        703,
        "cc2d59528a2141aa393f3447af74e210b0d8f8dabd8c7d902da1c1d1686d0788",
    ),
    (
        30,
        16,
        719,
        "513f43efe8275385845432b21a09bb8d08c6c91ba2f6ebbaf09612b94e6c4f32",
    ),
    (
        31,
        20,
        415,
        "aae4fab8051f757186afda9d528a931820a72b08b7c99179fa7dbfc5e5723a0e",
    ),
    (
        40,
        18,
        914,
        "faffc973840b61e3789bdc2c13deed124416aec38096f57172446f0fabdebf04",
    ),
    (
        44,
        25,
        681,
        "0906c57cf80e33c96a7100ae60e896d919e51a66d127e778f27af58c9679046d",
    ),
    (
        59,
        10,
        202,
        "b19d2529ac0c9441600c8e231c60b961381f24c3826ddda2b3d54f66996930ba",
    ),
    (
        64,
        10,
        180,
        "c3fec241bb8843f311ebf98df4418e8c0e88ad26feeb437ac7f2a87441216f79",
    ),
    (
        69,
        12,
        325,
        "58acb0df99eb27a51cef7c65934f12c61025a96e8093c2720379c0684cee31c6",
    ),
    (
        74,
        5,
        107,
        "60b8e116db2d367ae33374bfcbb65196e2bbd8cce21af1437ac9623bb2eedcac",
    ),
    (
        81,
        7,
        201,
        "745d2e6e22a37ef9b58e45096a6132acbbdf22c85a531491c7573fe222c63261",
    ),
    (
        84,
        7,
        197,
        "27990e0972d10340888d6e05c7d2eaa67f2889266e7f3e8f21d394822c7d5abd",
    ),
    (
        89,
        5,
        108,
        "ccdd457b000c43ac2a169746e60f5ed1289fd04f83591431168ad00b0fe65154",
    ),
    (
        104,
        3,
        77,
        "7822089cf4c9a03aff0ca5242972c0dee6f9052da0972fb894840b4d56750e20",
    ),
    (
        115,
        4,
        61,
        "b1841222c6e82ffd60696129aea39f6351cfac1a4cd8b50c4736de8ec0d86f9a",
    ),
    (
        124,
        3,
        148,
        "aa32ca19d6ed54f2fa7ce93f9b23dfd6a750bff6d4f3b4891069a84391af885c",
    ),
    (
        135,
        3,
        116,
        "8de9d4fc24776ea390c28be629cf6cefc347762e7587033d64b07e1be87d9903",
    ),
    (
        254,
        2,
        41,
        "85a82391f1666a44459d0db790821a96453aa3dfb7360f5d30a4617204187409",
    ),
];

#[test]
fn task22o46_real_ksr_matches_global_fixed_msvc_oracle_and_preserves_input() {
    let horizontal =
        super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
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
    let source_index = prelude.object.identity().0;
    let object_options = &traversal
        .resolved
        .objects
        .iter()
        .find(|object| object.source_object_index == source_index)
        .unwrap()
        .object;
    let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
    let before = snapshot(&horizontal.objects[0].records, &EXPECTED);
    let planned_before = plan.layers.clone();
    let object_before = object_options.clone();
    let nozzles_before = nozzles.clone();
    let region_options_before = EXPECTED
        .iter()
        .map(|&(layer, _, _, _)| {
            prelude
                .object
                .region_options(inputs[layer].as_ref().unwrap())
                .clone()
        })
        .collect::<Vec<_>>();

    let key_40 = &horizontal.objects[0].records[40]
        .as_ref()
        .unwrap()
        .fill_surfaces;
    for kind in [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::BottomBridge,
        RegionSurfaceKind::Internal,
        RegionSurfaceKind::InternalSolid,
    ] {
        assert!(
            key_40.iter().any(|surface| surface.as_parts().0 == kind),
            "key 40 is missing {kind:?}"
        );
    }
    assert!(
        key_40
            .iter()
            .filter(|surface| surface.as_parts().0 == RegionSurfaceKind::BottomBridge)
            .all(|surface| surface.as_parts().4 > 0.0)
    );

    let context = KsrRun {
        horizontal,
        inputs,
        layers: &plan.layers,
        object: object_options,
        nozzles,
        scale: traversal.scale,
    };
    let first = run(context);
    let second = run(context);
    assert_eq!(first, second);
    assert_eq!(snapshot(&horizontal.objects[0].records, &EXPECTED), before);
    assert_eq!(plan.layers, planned_before);
    assert_eq!(object_options, &object_before);
    assert_eq!(nozzles, &nozzles_before);
    assert_eq!(
        EXPECTED
            .iter()
            .map(|&(layer, _, _, _)| prelude
                .object
                .region_options(inputs[layer].as_ref().unwrap())
                .clone())
            .collect::<Vec<_>>(),
        region_options_before
    );

    let mut aggregate = Vec::new();
    let mut path_total = 0;
    let mut point_total = 0;
    for ((layer, paths, points, digest), bytes) in EXPECTED.iter().zip(&first) {
        assert_eq!(path_count(bytes), *paths, "layer {layer}");
        assert_eq!(point_count(bytes), *points, "layer {layer}");
        assert_eq!(sha256(bytes), *digest, "layer {layer}");
        path_total += *paths;
        point_total += *points;
        aggregate.extend_from_slice(bytes);
    }
    assert_eq!((path_total, point_total), (186, 5_941));
    assert_eq!(
        sha256(&aggregate),
        "917adc6ea02ad7cd7af79e45d90db6f4c1497bf5c8716d7f2f49b7de4b2070ef"
    );

    external_surfaces::dispose(external);
}

#[derive(Clone, Copy)]
struct KsrRun<'a> {
    horizontal: &'a crate::project_slice::prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
    inputs: &'a [Option<crate::project_slice::perimeters::types::PerimeterInputRecord>],
    layers: &'a [crate::project_slice::layers::PlannedLayer],
    object: &'a crate::ObjectOptions,
    nozzles: &'a crate::OrcaFloats,
    scale: crate::geometry::CoordinateScale,
}

fn run(context: KsrRun<'_>) -> Vec<Vec<u8>> {
    EXPECTED
        .iter()
        .map(|&(layer, _, _, _)| {
            let input = context.inputs[layer].as_ref().unwrap();
            let surfaces = &context.horizontal.objects[0].records[layer]
                .as_ref()
                .unwrap()
                .fill_surfaces;
            let paths = generate_sparse_infill_polylines_for_anchoring(SparseAnchoringLayer {
                planned: &context.layers[layer],
                fill_surfaces: surfaces,
                region_options: context.horizontal.predecessor.objects[0]
                    .predecessor
                    .predecessor
                    .predecessor
                    .predecessor
                    .object
                    .region_options(input),
                object_options: context.object,
                nozzle_diameters: context.nozzles,
                scale: context.scale,
            })
            .unwrap();
            serialize(layer, &paths)
        })
        .collect()
}

fn serialize(layer: usize, paths: &[Polyline]) -> Vec<u8> {
    let mut output = format!("layer {layer} polylines {}\n", paths.len());
    for (index, path) in paths.iter().enumerate() {
        output.push_str(&format!("polyline {index} points {}", path.points().len()));
        for point in path.points() {
            output.push_str(&format!(" {},{}", point.x(), point.y()));
        }
        output.push('\n');
    }
    output.into_bytes()
}

fn snapshot(
    records: &[Option<crate::project_slice::prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord>],
    expected: &[(usize, usize, usize, &str)],
) -> Vec<u8> {
    let mut output = Vec::new();
    for &(layer, _, _, _) in expected {
        let record = records[layer].as_ref().unwrap();
        for surface in &record.fill_surfaces {
            let (kind, expolygon, thickness, thickness_layers, angle, extra) = surface.as_parts();
            output.extend_from_slice(
                format!("{kind:?}:{thickness:?}:{thickness_layers}:{angle:?}:{extra}|").as_bytes(),
            );
            encode_expolygon(&mut output, expolygon);
        }
    }
    output
}

fn encode_expolygon(output: &mut Vec<u8>, expolygon: &crate::geometry::ExPolygon) {
    for polygon in std::iter::once(expolygon.contour()).chain(expolygon.holes()) {
        for point in polygon.points() {
            output.extend_from_slice(format!("{},{};", point.x(), point.y()).as_bytes());
        }
        output.push(b'|');
    }
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
}

fn path_count(bytes: &[u8]) -> usize {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .split_whitespace()
        .nth(3)
        .unwrap()
        .parse()
        .unwrap()
}

fn point_count(bytes: &[u8]) -> usize {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .skip(1)
        .map(|line| {
            line.split_whitespace()
                .nth(3)
                .unwrap()
                .parse::<usize>()
                .unwrap()
        })
        .sum()
}
