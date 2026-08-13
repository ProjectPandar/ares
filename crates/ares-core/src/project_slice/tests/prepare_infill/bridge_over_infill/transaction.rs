use std::fmt::Write;

use sha2::{Digest, Sha256};

use crate::{
    SliceError,
    project_slice::{
        prepare_infill::{
            bridge_over_infill::{PreparedPostBridgeCandidates, transaction},
            external_surfaces,
        },
        region_slices::RegionSurfaceKind,
        tests::support::KsrArchive,
    },
};

mod options;

#[test]
fn task22o71_real_ksr_commits_first_internal_bridge_layer() {
    let raw = super::prepare(KsrArchive::new());
    assert!(raw.objects[0].surfaces_by_layer.contains_key(&15));
    assert!(
        raw.predecessor.predecessor.objects[0].records[15]
            .as_ref()
            .unwrap()
            .fill_surfaces
            .iter()
            .all(|surface| surface.as_parts().0 != RegionSurfaceKind::InternalBridge)
    );

    let prepared = transaction::prepare(raw).unwrap();
    assert!(
        prepared.predecessor.predecessor.objects[0].records[15]
            .as_ref()
            .unwrap()
            .fill_surfaces
            .iter()
            .any(|surface| {
                let (kind, expolygon, ..) = surface.as_parts();
                kind == RegionSurfaceKind::InternalBridge && expolygon.area() > 0.0
            })
    );

    transaction::dispose(prepared);
}

#[test]
fn task22o71_real_ksr_committed_surface_snapshot_is_repeatable() {
    let prepared = transaction::prepare(super::prepare(KsrArchive::new())).unwrap();
    let first = snapshot(&prepared);
    transaction::dispose(prepared);

    let prepared = transaction::prepare(super::prepare(KsrArchive::new())).unwrap();
    let second = snapshot(&prepared);
    transaction::dispose(prepared);

    assert_eq!(second.bridge_layers, first.bridge_layers,);
    assert_eq!(second.bridge_surfaces, first.bridge_surfaces);
    assert_eq!(
        second.bridge_expolygon_points,
        first.bridge_expolygon_points
    );
    assert_eq!(second.bytes, first.bytes);
    assert_eq!(
        first.bridge_layers,
        vec![
            15, 30, 31, 41, 45, 60, 65, 70, 75, 82, 85, 90, 105, 116, 125, 136, 255,
        ]
    );
    assert_eq!(
        (
            first.bridge_surfaces,
            first.bridge_expolygon_points,
            sha256(&first.bytes),
        ),
        (
            47,
            15_689,
            "c547cb34b8d5d27d572a166f13a16741f75f7f9d34f15db59ddac8575b5a33b9".to_owned(),
        )
    );
}

#[test]
fn task22o71_empty_candidate_inventory_preserves_every_surface_byte() {
    let mut raw = super::prepare(KsrArchive::new());
    raw.objects[0].surfaces_by_layer.clear();
    let before = snapshot_horizontal(&raw.predecessor.predecessor);

    let prepared = transaction::prepare(raw).unwrap();
    let after = snapshot(&prepared);

    assert_eq!(after.bytes, before.bytes);
    assert_eq!(after.bridge_layers, before.bridge_layers);
    assert_eq!(after.bridge_surfaces, 0);
    transaction::dispose(prepared);
}

#[test]
fn task22o71_nofilter_empty_candidate_accepts_an_absent_lower_record() {
    let mut raw = super::prepare(KsrArchive::new());
    let mut candidate = raw.objects[0]
        .surfaces_by_layer
        .remove(&15)
        .unwrap()
        .remove(0);
    candidate.new_polygons.clear();
    raw.objects[0].surfaces_by_layer.clear();
    raw.objects[0].surfaces_by_layer.insert(15, vec![candidate]);
    raw.predecessor.predecessor.objects[0].records[14] = None;
    raw.predecessor.predecessor.predecessor.objects[0]
        .predecessor
        .predecessor
        .predecessor
        .predecessor
        .object
        .records[14] = None;

    let prepared = transaction::prepare(raw).unwrap();

    assert!(prepared.predecessor.predecessor.objects[0].records[14].is_none());
    transaction::dispose(prepared);
}

#[test]
fn task22o71_unported_anchor_surface_kinds_fail_without_panicking_or_fallback() {
    for kind in [
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::InternalBridge,
        RegionSurfaceKind::InternalVoid,
    ] {
        let mut raw = super::prepare(KsrArchive::new());
        let surfaces = &mut raw.predecessor.predecessor.objects[0].records[14]
            .as_mut()
            .unwrap()
            .fill_surfaces;
        surfaces.push(surfaces[0].clone_with_kind(kind));
        assert_unsupported_raw(raw, "bridge_over_infill_anchor_surface_kind");
    }
}

#[test]
fn task22o71_late_rewrite_error_disposes_owned_graph_without_successor() {
    external_surfaces::reset_hooks();
    transaction::reset_hooks();
    let mut raw = super::prepare(KsrArchive::new());
    raw.predecessor.predecessor.objects[0].records[255]
        .as_mut()
        .unwrap()
        .fill_surfaces
        .push(crate::project_slice::region_slices::RegionSurface::new(
            RegionSurfaceKind::Bottom,
            super::outside_clipper_range(),
        ));

    let error = match transaction::prepare(raw) {
        Ok(prepared) => {
            transaction::dispose(prepared);
            panic!("invalid upper ensuring geometry must not publish a successor")
        }
        Err(error) => error,
    };

    assert_eq!(
        error,
        SliceError::InvalidInput(
            "bridge-over-infill coordinate is outside the supported Clipper range".to_owned(),
        )
    );
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 0);
    assert_eq!(external_surfaces::disposals(), 1);
    external_surfaces::reset_hooks();
    transaction::reset_hooks();
}

fn assert_unsupported_mutation(from: &str, to: &str, expected: &str) {
    let mut archive = KsrArchive::new();
    archive.replace_unique("Metadata/project_settings.config", from, to);
    let raw = super::prepare(archive);
    assert_unsupported_raw(raw, expected);
}

fn assert_error_mutation(from: &str, to: &str, expected: SliceError) {
    let mut archive = KsrArchive::new();
    archive.replace_unique("Metadata/project_settings.config", from, to);
    assert_error_raw(super::prepare(archive), expected);
}

fn assert_unsupported_raw(raw: PreparedPostBridgeCandidates, expected: &str) {
    assert_error_raw(
        raw,
        SliceError::UnsupportedProjectFeature(expected.to_owned()),
    );
}

fn assert_error_raw(raw: PreparedPostBridgeCandidates, expected: SliceError) {
    external_surfaces::reset_hooks();
    transaction::reset_hooks();
    assert_eq!(external_surfaces::disposals(), 0);

    let error = match transaction::prepare(raw) {
        Ok(prepared) => {
            transaction::dispose(prepared);
            panic!("deferred bridge behavior must not use a fallback")
        }
        Err(error) => error,
    };

    assert_eq!(error, expected);
    assert_eq!(transaction::invocations(), 1);
    assert_eq!(transaction::disposals(), 0);
    assert_eq!(external_surfaces::disposals(), 1);
    external_surfaces::reset_hooks();
    transaction::reset_hooks();
}

struct SurfaceSnapshot {
    bytes: Vec<u8>,
    bridge_layers: Vec<usize>,
    bridge_surfaces: usize,
    bridge_expolygon_points: usize,
}

fn snapshot(prepared: &transaction::PreparedPostBridgeOverInfill) -> SurfaceSnapshot {
    snapshot_horizontal(&prepared.predecessor.predecessor)
}

fn snapshot_horizontal(
    horizontal: &crate::project_slice::prepare_infill::horizontal_shell_propagation::PreparedPostHorizontalShellPropagation,
) -> SurfaceSnapshot {
    let mut bytes = Vec::new();
    let mut bridge_layers = Vec::new();
    let mut bridge_surfaces = 0;
    let mut bridge_expolygon_points = 0;
    put_usize(&mut bytes, horizontal.objects.len());
    for object in &horizontal.objects {
        put_usize(&mut bytes, object.records.len());
        for (layer_index, record) in object.records.iter().enumerate() {
            let Some(record) = record else {
                bytes.push(0);
                continue;
            };
            bytes.push(1);
            put_usize(&mut bytes, record.fill_surfaces.len());
            let mut layer_has_bridge = false;
            for surface in &record.fill_surfaces {
                let bridge_points = put_surface(&mut bytes, surface);
                layer_has_bridge |= bridge_points.is_some();
                bridge_surfaces += usize::from(bridge_points.is_some());
                bridge_expolygon_points += bridge_points.unwrap_or(0);
            }
            if layer_has_bridge {
                bridge_layers.push(layer_index);
            }
        }
    }
    SurfaceSnapshot {
        bytes,
        bridge_layers,
        bridge_surfaces,
        bridge_expolygon_points,
    }
}

fn put_surface(
    output: &mut Vec<u8>,
    surface: &crate::project_slice::region_slices::RegionSurface,
) -> Option<usize> {
    let (kind, expolygon, thickness, layers, angle, extra_perimeters) = surface.as_parts();
    output.push(kind as u8);
    output.extend_from_slice(&thickness.to_bits().to_le_bytes());
    output.extend_from_slice(&layers.to_le_bytes());
    output.extend_from_slice(&angle.to_bits().to_le_bytes());
    output.extend_from_slice(&extra_perimeters.to_le_bytes());
    put_polygon(output, expolygon.contour());
    put_usize(output, expolygon.holes().len());
    for hole in expolygon.holes() {
        put_polygon(output, hole);
    }
    (kind == RegionSurfaceKind::InternalBridge).then(|| {
        expolygon.contour().points().len()
            + expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().len())
                .sum::<usize>()
    })
}

fn put_polygon(output: &mut Vec<u8>, polygon: &crate::geometry::Polygon) {
    put_usize(output, polygon.points().len());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

fn put_usize(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&(value as u64).to_le_bytes());
}

fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut output, byte| {
            write!(output, "{byte:02x}").unwrap();
            output
        })
}
