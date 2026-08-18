use crate::{
    geometry::Polyline,
    project_slice::{
        prepare_infill::{
            bridge_over_infill::sparse_anchoring::generate_sparse_infill_polylines_for_anchoring,
            external_surfaces,
        },
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};

const TARGET_LAYERS: [usize; 18] = [
    14, 29, 30, 31, 40, 44, 59, 64, 69, 74, 81, 84, 89, 104, 115, 124, 135, 254,
];

#[test]
fn task22o75_real_ksr_sparse_anchors_are_repeatable_and_preserve_input() {
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
    let before = snapshot(&horizontal.objects[0].records, &TARGET_LAYERS);
    let planned_before = plan.layers.clone();
    let object_before = object_options.clone();
    let nozzles_before = nozzles.clone();
    let region_options_before = TARGET_LAYERS
        .iter()
        .map(|&layer| {
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
        external: &external,
    };
    let first = run(context);
    let second = run(context);
    assert_eq!(first, second);
    assert_eq!(
        snapshot(&horizontal.objects[0].records, &TARGET_LAYERS),
        before
    );
    assert_eq!(plan.layers, planned_before);
    assert_eq!(object_options, &object_before);
    assert_eq!(nozzles, &nozzles_before);
    assert_eq!(
        TARGET_LAYERS
            .iter()
            .map(|&layer| prelude
                .object
                .region_options(inputs[layer].as_ref().unwrap())
                .clone())
            .collect::<Vec<_>>(),
        region_options_before
    );

    for (&layer, bytes) in TARGET_LAYERS.iter().zip(&first) {
        let paths = path_count(bytes);
        assert!(paths > 0, "layer {layer}");
        assert!(point_count(bytes) >= paths * 2, "layer {layer}");
    }

    external_surfaces::dispose(external);
}

#[derive(Clone, Copy)]
struct KsrRun<'a> {
    external:
        &'a crate::project_slice::prepare_infill::external_surfaces::PreparedPostExternalSurfaces,
}

fn run(context: KsrRun<'_>) -> Vec<Vec<u8>> {
    TARGET_LAYERS
        .iter()
        .map(|&layer| {
            let paths =
                generate_sparse_infill_polylines_for_anchoring(context.external, 0, layer).unwrap();
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
    layers: &[usize],
) -> Vec<u8> {
    let mut output = Vec::new();
    for &layer in layers {
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
