//! Public 3MF replay of `GCode.cpp::needs_retraction` Tree base containment.

mod support_events;

use support_events::{Mutation, mutate, support_events};

const ROOF_BASE_TALL_SKIRT: &[(&str, &str)] = &[
    ("skirt_height", "40"),
    ("support_top_z_distance", "0"),
    ("support_filament", "0"),
    ("support_interface_filament", "1"),
];

#[test]
fn single_cpu_slim_tree_concentric_hole_preserves_extrusion_direction() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case = super::overhang_case(
        "tree-support/slim/concentric-hole-direction",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    )
    .unwrap();
    let native = support_events(std::str::from_utf8(&case.reference).unwrap());
    assert!(
        native
            .iter()
            .map(|events| events.closed_loops)
            .sum::<usize>()
            > 0,
        "native reference must extrude support loops closed"
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let ares = support_events(std::str::from_utf8(&output).unwrap());
    assert_eq!(
        ares.iter().map(|events| events.closed_loops).sum::<usize>(),
        native
            .iter()
            .map(|events| events.closed_loops)
            .sum::<usize>(),
        "Concentric hole traversal must preserve source extrusion direction"
    );
    let inside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Loop,
        true,
    ));
    let outside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Loop,
        false,
    ));
    assert_ne!(
        inside
            .iter()
            .map(|events| events.closed_loops)
            .sum::<usize>(),
        ares.iter().map(|events| events.closed_loops).sum::<usize>(),
        "in-scope mutation must flip closed_loops"
    );
    assert_eq!(
        outside
            .iter()
            .map(|events| events.closed_loops)
            .sum::<usize>(),
        ares.iter().map(|events| events.closed_loops).sum::<usize>(),
        "outside mutation must not flip closed_loops"
    );
}

#[test]
fn single_cpu_slim_tree_roof_slice_does_not_suppress_retraction() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case = super::overhang_case(
        "tree-support/slim/roof-slice-retraction",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    )
    .unwrap();
    let native = support_events(std::str::from_utf8(&case.reference).unwrap());
    assert!(
        native
            .iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>()
            > 0,
        "native reference must retract out of support interfaces"
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let ares = support_events(std::str::from_utf8(&output).unwrap());
    assert_eq!(
        ares.iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        native
            .iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        "Tree roof planner slice must not exempt support travel from retraction"
    );
    let inside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Wipe,
        true,
    ));
    let outside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Wipe,
        false,
    ));
    assert_ne!(
        inside
            .iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        ares.iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        "in-scope mutation must flip interface_wipes"
    );
    assert_eq!(
        outside
            .iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        ares.iter()
            .map(|events| events.interface_wipes)
            .sum::<usize>(),
        "outside mutation must not flip interface_wipes"
    );
}

#[test]
fn single_cpu_slim_tree_roof_base_tall_skirt_preserves_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    super::compare_overhang(
        "tree-support/slim/roof-base-tall-skirt",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    );
}

#[test]
fn single_cpu_slim_tree_tall_skirt_preserves_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    super::compare_overhang(
        "tree-support/slim/tall-skirt",
        "tree_slim",
        &[("skirt_height", "40")],
    );
}

#[test]
fn single_cpu_slim_tree_contained_travel_does_not_retract() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case =
        super::overhang_case("tree-support/slim/contained-travel", "tree_slim", &[]).unwrap();
    let native = support_events(std::str::from_utf8(&case.reference).unwrap());
    assert!(
        native
            .iter()
            .map(|events| events.contained_travels)
            .sum::<usize>()
            > 0,
        "native reference must travel inside one support base"
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    let ares = support_events(std::str::from_utf8(&output).unwrap());
    assert_eq!(
        ares.iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        native
            .iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        "Tree travel contained in one base area must not retract or lift"
    );
    let inside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Travel,
        true,
    ));
    let outside = support_events(&mutate(
        std::str::from_utf8(&output).unwrap(),
        Mutation::Travel,
        false,
    ));
    assert_ne!(
        inside
            .iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        ares.iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        "in-scope mutation must flip contained_travels"
    );
    assert_eq!(
        outside
            .iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        ares.iter()
            .map(|events| events.contained_travels)
            .sum::<usize>(),
        "outside mutation must not flip contained_travels"
    );
}
