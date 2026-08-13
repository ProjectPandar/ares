use crate::project_slice::{
    prepare_infill::{bridge_over_infill, external_surfaces},
    tests::support::KsrArchive,
};

use super::horizontal_shell_propagation;
use bridge_over_infill::internal_bridge_angle::apply_internal_bridge_angle_override;

const ROOT_TRANSFORM: &str = r#"transform="1 0 0 0 1 0 0 0 1 133.039205 115.992105 46""#;
const ROTATED_TRANSFORM: &str = r#"transform="0 1 0 -1 0 0 0 0 1 133.039205 115.992105 46""#;

#[test]
fn task22o49_real_ksr_default_override_preserves_all_candidate_angle_bits_and_input() {
    let prepared = prepare(KsrArchive::new());
    let before = snapshot(&prepared);
    let region_before = first_region(&prepared).clone();

    let first = output_bits(&prepared);
    let second = output_bits(&prepared);

    assert_eq!(first, vec![0; 43]);
    assert_eq!(second, first);
    assert_eq!(snapshot(&prepared), before);
    assert_eq!(first_region(&prepared), &region_before);
    bridge_over_infill::dispose(prepared);
}

#[test]
fn task22o49_real_ksr_absolute_alignment_adds_retained_occurrence_rotation() {
    let prepared = prepare(mutated_archive(false));
    let before = snapshot(&prepared);
    let region_before = first_region(&prepared).clone();
    let contexts = contexts(&prepared);

    assert!(contexts.iter().all(|context| {
        context.internal_bridge_angle_bits == 17.3_f64.to_bits()
            && !context.relative
            && context.align
            && context.model_rotation_bits == std::f64::consts::FRAC_PI_2.to_bits()
    }));
    let first = output_bits(&prepared);
    let second = output_bits(&prepared);
    assert_eq!(first, vec![0x3ffd_f6bc_6c24_e864; 43]);
    assert_eq!(second, first);
    assert_eq!(snapshot(&prepared), before);
    assert_eq!(first_region(&prepared), &region_before);
    bridge_over_infill::dispose(prepared);
}

#[test]
fn task22o49_real_ksr_relative_override_ignores_the_same_nonzero_rotation() {
    let prepared = prepare(mutated_archive(true));
    let before = snapshot(&prepared);
    let region_before = first_region(&prepared).clone();
    let contexts = contexts(&prepared);

    assert!(contexts.iter().all(|context| {
        context.internal_bridge_angle_bits == 17.3_f64.to_bits()
            && context.relative
            && context.align
            && context.model_rotation_bits == std::f64::consts::FRAC_PI_2.to_bits()
    }));
    let first = output_bits(&prepared);
    let second = output_bits(&prepared);
    assert_eq!(first, vec![0x3fd3_5304_5f82_ed32; 43]);
    assert_eq!(second, first);
    assert_eq!(snapshot(&prepared), before);
    assert_eq!(first_region(&prepared), &region_before);
    bridge_over_infill::dispose(prepared);
}

fn prepare(archive: KsrArchive) -> bridge_over_infill::PreparedPostBridgeCandidates {
    let horizontal = horizontal_shell_propagation::fixture::prepare(archive.bytes());
    let external = external_surfaces::prepare(horizontal).unwrap();
    bridge_over_infill::prepare(external).unwrap()
}

fn mutated_archive(relative: bool) -> KsrArchive {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        r#""internal_bridge_angle": "0""#,
        r#""internal_bridge_angle": "17.3""#,
    );
    archive.replace_unique(
        "Metadata/project_settings.config",
        r#""align_infill_direction_to_model": "0""#,
        r#""align_infill_direction_to_model": "1""#,
    );
    if relative {
        archive.replace_unique(
            "Metadata/project_settings.config",
            r#""relative_bridge_angle": "0""#,
            r#""relative_bridge_angle": "1""#,
        );
    }
    archive.replace_unique("3D/3dmodel.model", ROOT_TRANSFORM, ROTATED_TRANSFORM);
    archive
}

#[derive(Clone, Copy)]
struct ContextBits {
    internal_bridge_angle_bits: u64,
    relative: bool,
    align: bool,
    model_rotation_bits: u64,
}

fn contexts(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<ContextBits> {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal = &horizontal.predecessor;
    let traversal_object = &traversal.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();

    prepared.objects[0]
        .surfaces_by_layer
        .values()
        .flatten()
        .map(|candidate| {
            let input = inputs[candidate.source.layer_index].as_ref().unwrap();
            let region = prelude.object.region_options(input);
            ContextBits {
                internal_bridge_angle_bits: region.internal_bridge_angle.0.to_bits(),
                relative: region.relative_bridge_angle.0,
                align: region.align_infill_direction_to_model.0,
                model_rotation_bits: input.model_rotation_rad.to_bits(),
            }
        })
        .collect()
}

fn output_bits(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<u64> {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal_object = &horizontal.predecessor.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();

    prepared.objects[0]
        .surfaces_by_layer
        .values()
        .flatten()
        .map(|candidate| {
            let input = inputs[candidate.source.layer_index].as_ref().unwrap();
            apply_internal_bridge_angle_override(
                candidate.bridge_angle,
                prelude.object.region_options(input),
                input.model_rotation_rad,
            )
            .to_bits()
        })
        .collect()
}

fn first_region(
    prepared: &bridge_over_infill::PreparedPostBridgeCandidates,
) -> &crate::RegionOptions {
    let horizontal = &prepared.predecessor.predecessor;
    let traversal_object = &horizontal.predecessor.objects[0];
    let prelude = &traversal_object
        .predecessor
        .predecessor
        .predecessor
        .predecessor;
    let (_, inputs) = prelude.object.as_parts();
    prelude
        .object
        .region_options(inputs.iter().flatten().next().unwrap())
}

fn snapshot(prepared: &bridge_over_infill::PreparedPostBridgeCandidates) -> Vec<u64> {
    let mut bits = Vec::new();
    for (&layer_index, candidates) in &prepared.objects[0].surfaces_by_layer {
        bits.push(layer_index as u64);
        bits.push(candidates.len() as u64);
        for candidate in candidates {
            bits.extend([
                candidate.source.layer_index as u64,
                candidate.source.region_index as u64,
                candidate.source.surface_index as u64,
                candidate.bridge_angle.to_bits(),
                candidate.new_polygons.len() as u64,
            ]);
            for polygon in &candidate.new_polygons {
                bits.push(polygon.points().len() as u64);
                bits.extend(
                    polygon
                        .points()
                        .iter()
                        .flat_map(|point| [point.x() as u64, point.y() as u64]),
                );
            }
        }
    }
    for context in contexts(prepared) {
        bits.extend([
            context.internal_bridge_angle_bits,
            u64::from(context.relative),
            u64::from(context.align),
            context.model_rotation_bits,
        ]);
    }
    bits
}
