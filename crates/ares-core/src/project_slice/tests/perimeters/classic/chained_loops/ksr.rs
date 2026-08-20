use crate::project_slice::perimeters::{
    classic::{
        chained_loops::{ChainedLoopNode, ExtrusionLoopRole},
        materialize::ExtrusionRole,
        traversal::{PendingLoopRole, PendingPathBranch, TraversalSeed},
    },
    prepare_post_classic_chained_loops,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o8_ksr_preserves_full_o5_alignment_roles_and_branch_provenance() {
    let prepared = prepare_post_classic_chained_loops(ksr_project()).unwrap();
    assert_eq!(prepared.objects.len(), prepared.predecessor.objects.len());
    let mut role_counts = [0_usize; 3];
    let mut some_count = 0;
    let mut none_count = 0;
    let mut saw_ordinary = false;
    let mut saw_overhang = false;
    for (output_object, traversal_object) in
        prepared.objects.iter().zip(&prepared.predecessor.objects)
    {
        assert_eq!(output_object.records.len(), traversal_object.records.len());
        for (output_record, traversal_record) in
            output_object.records.iter().zip(&traversal_object.records)
        {
            assert_eq!(output_record.is_some(), traversal_record.is_some());
            let (Some(output_record), Some(traversal_record)) = (output_record, traversal_record)
            else {
                continue;
            };
            saw_ordinary |= matches!(
                traversal_record.branch,
                PendingPathBranch::OrdinaryUnsplit { .. }
            );
            saw_overhang |= matches!(
                traversal_record.branch,
                PendingPathBranch::OverhangClipping { .. }
            );
            assert_eq!(
                output_record.surfaces.len(),
                traversal_record.surfaces.len()
            );
            for (output_surface, traversal_surface) in output_record
                .surfaces
                .iter()
                .zip(&traversal_record.surfaces)
            {
                assert_eq!(output_surface.source_index, traversal_surface.source_index);
                assert_nodes(
                    &output_surface.roots,
                    &traversal_surface.roots,
                    &mut role_counts,
                    &mut some_count,
                    &mut none_count,
                );
            }
        }
    }
    assert!(saw_ordinary && saw_overhang);
    assert!(some_count > 0);
    assert_eq!(role_counts.iter().sum::<usize>(), some_count);
    assert!(role_counts.iter().all(|count| *count > 0));
    let _fixture_may_have_no_empty_clipped_nodes = none_count;
}

#[test]
fn task22o8_ksr_loop_path_fields_are_nonempty_and_deterministic() {
    let first = checksum();
    assert_ne!(first, 0);
    assert_eq!(first, checksum());
}

fn assert_nodes(
    nodes: &[ChainedLoopNode],
    seeds: &[TraversalSeed],
    role_counts: &mut [usize; 3],
    some_count: &mut usize,
    none_count: &mut usize,
) {
    assert_eq!(nodes.len(), seeds.len());
    let mut pending = nodes.iter().zip(seeds).collect::<Vec<_>>();
    while let Some((node, seed)) = pending.pop() {
        match &node.extrusion_loop {
            Some(extrusion_loop) => {
                *some_count += 1;
                let index = match extrusion_loop.role {
                    ExtrusionLoopRole::Internal => 0,
                    ExtrusionLoopRole::Default => 1,
                    ExtrusionLoopRole::Hole => 2,
                };
                role_counts[index] += 1;
                assert_eq!(
                    extrusion_loop.role,
                    match seed.loop_role {
                        PendingLoopRole::Internal => ExtrusionLoopRole::Internal,
                        PendingLoopRole::Default => ExtrusionLoopRole::Default,
                        PendingLoopRole::Hole => ExtrusionLoopRole::Hole,
                    }
                );
            }
            None => *none_count += 1,
        }
        assert_eq!(node.children.len(), seed.children.len());
        pending.extend(node.children.iter().zip(&seed.children));
    }
}

fn checksum() -> i128 {
    let prepared = prepare_post_classic_chained_loops(ksr_project()).unwrap();
    let mut checksum = 0_i128;
    for object in &prepared.objects {
        mix(&mut checksum, object.records.len() as i128);
        for record in &object.records {
            mix(&mut checksum, i128::from(record.is_some()));
            if let Some(record) = record {
                accumulate_record(record, &mut checksum);
            }
        }
    }
    checksum
}

fn accumulate_record(
    record: &crate::project_slice::perimeters::classic::chained_loops::TestPreparedChainedLoopRecord,
    checksum: &mut i128,
) {
    mix(checksum, record.surfaces.len() as i128);
    for surface in &record.surfaces {
        mix(checksum, surface.source_index as i128);
        accumulate_surface(&surface.roots, checksum);
    }
}

fn accumulate_surface(roots: &[ChainedLoopNode], checksum: &mut i128) {
    let mut pending = roots.iter().rev().collect::<Vec<_>>();
    while let Some(node) = pending.pop() {
        mix(
            checksum,
            node.extrusion_loop
                .as_ref()
                .map_or(-1, |loop_| match loop_.role {
                    ExtrusionLoopRole::Internal => 1,
                    ExtrusionLoopRole::Default => 2,
                    ExtrusionLoopRole::Hole => 3,
                }),
        );
        mix(checksum, node.children.len() as i128);
        if let Some(extrusion_loop) = &node.extrusion_loop {
            accumulate_loop(extrusion_loop, checksum);
        }
        pending.extend(node.children.iter().rev());
    }
}

fn accumulate_loop(
    extrusion_loop: &crate::project_slice::perimeters::classic::chained_loops::ExtrusionLoop,
    checksum: &mut i128,
) {
    mix(checksum, extrusion_loop.paths.len() as i128);
    for path in &extrusion_loop.paths {
        mix(
            checksum,
            match path.role {
                ExtrusionRole::ExternalPerimeter => 1,
                ExtrusionRole::Perimeter => 2,
                ExtrusionRole::OverhangPerimeter => 3,
                ExtrusionRole::GapFill => 4,
                ExtrusionRole::SolidInfill => 5,
            },
        );
        mix(checksum, i128::from(path.mm3_per_mm.to_bits()));
        mix(checksum, i128::from(path.width.to_bits()));
        mix(checksum, i128::from(path.height.to_bits()));
        mix(checksum, path.polyline.points.len() as i128);
        for point in &path.polyline.points {
            mix(checksum, i128::from(point.x));
            mix(checksum, i128::from(point.y));
            mix(checksum, i128::from(point.z));
        }
    }
}

fn mix(checksum: &mut i128, value: i128) {
    *checksum = checksum.wrapping_mul(257).wrapping_add(value);
}
