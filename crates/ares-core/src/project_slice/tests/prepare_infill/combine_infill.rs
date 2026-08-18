mod gates;
mod lifecycle;

use crate::project_slice::{
    prepare_infill::{
        bridge_over_infill::transaction::{self, PreparedPostBridgeOverInfill},
        combine_infill,
    },
    tests::{prepare_infill::bridge_over_infill::transaction::snapshot, support::KsrArchive},
};

#[test]
fn task22o72_real_ksr_identity_retains_deep_o71_graph_and_full_topology_bytes() {
    combine_infill::reset_hooks();
    transaction::reset_hooks();
    let input = prepare_o71(KsrArchive::new());
    assert_eq!(materialized_combination_options(&input), [(false, 15.0)]);
    let deep_predecessor = std::ptr::from_ref(input.predecessor.predecessor.predecessor.as_ref());
    let before = snapshot(&input);

    let output = combine_infill::prepare(input).unwrap();
    let after = snapshot(&output.predecessor);

    assert_eq!(
        std::ptr::from_ref(
            output
                .predecessor
                .predecessor
                .predecessor
                .predecessor
                .as_ref(),
        ),
        deep_predecessor
    );
    assert_eq!(after.bytes, before.bytes);
    assert_eq!(after.bridge_layers, before.bridge_layers);
    assert_eq!(after.bridge_surfaces, before.bridge_surfaces);
    assert_eq!(
        after.bridge_expolygon_points,
        before.bridge_expolygon_points
    );
    assert_eq!(combine_infill::invocations(), 1);
    assert_eq!(combine_infill::disposals(), 0);

    combine_infill::dispose(output);
    assert_eq!(combine_infill::disposals(), 1);
    assert_eq!(transaction::disposals(), 1);
    combine_infill::reset_hooks();
    transaction::reset_hooks();
}

#[test]
fn task22o72_real_ksr_identity_is_repeatable() {
    let first = combine_infill::prepare(prepare_o71(KsrArchive::new())).unwrap();
    let first_snapshot = snapshot(&first.predecessor);
    combine_infill::dispose(first);

    let second = combine_infill::prepare(prepare_o71(KsrArchive::new())).unwrap();
    let second_snapshot = snapshot(&second.predecessor);
    combine_infill::dispose(second);

    assert_eq!(second_snapshot.bytes, first_snapshot.bytes);
}

pub(super) fn prepare_o71(archive: KsrArchive) -> PreparedPostBridgeOverInfill {
    transaction::prepare(super::bridge_over_infill::prepare(archive)).unwrap()
}

pub(super) fn materialized_combination_options(
    prepared: &PreparedPostBridgeOverInfill,
) -> Vec<(bool, f64)> {
    let traversal = &prepared.predecessor.predecessor.predecessor;
    traversal
        .objects
        .iter()
        .flat_map(|object| {
            let prelude = &object.predecessor.predecessor.predecessor.predecessor;
            let (compensated, _) = prelude.object.as_parts();
            let (post_regions, _) = compensated.as_parts();
            let (_, _, regions) = post_regions.as_parts();
            regions.iter().map(|region| {
                let options = region.as_parts().1;
                (
                    options.infill_combination.0,
                    options.sparse_infill_density.0,
                )
            })
        })
        .collect()
}
