use std::{cell::Cell, fmt::Debug};

use crate::SliceError;
use crate::{Transform3d, project::effective_config::types::ResolvedPrintObjectConfig};

use super::super::{
    layers::{LayerBudget, LayerPair, PlannedLayer, generate_layer_pairs, planned_layers},
    parameters::SlicingParameters,
    plan_resolved_objects,
    profile::fixed_layer_height_profile,
};
use super::support::{object_options, resolved};

const LIMIT_ERROR: &str = "project layer count exceeds supported limit of 100000";
const PROGRESS_ERROR: &str = "layer_height does not advance print_z";

#[test]
fn task22a_fixed_profile_compresses_equal_and_preserves_unequal_heights() {
    assert_eq!(
        fixed_layer_height_profile(&parameters(0.2, 0.2, 0.1, 1.0)),
        vec![0.0, 0.2, 1.0, 0.2]
    );
    assert_eq!(
        fixed_layer_height_profile(&parameters(0.15, 0.2, 0.1, 1.0)),
        vec![0.0, 0.15, 0.15, 0.15, 0.15, 0.2, 1.0, 0.2]
    );
}

#[test]
fn task22a_fixed_profile_uses_strict_epsilon_and_unrounded_top() {
    let epsilon = 0.0001_f64;
    let below = f64::from_bits(epsilon.to_bits() - 1);
    let top = f64::from_bits(1.0_f64.to_bits() + 1);
    let exact = fixed_layer_height_profile(&parameters(epsilon, 0.0, 0.1, top));
    assert_eq!(
        exact,
        vec![0.0, epsilon, epsilon, epsilon, epsilon, 0.0, top, 0.0]
    );
    let compressed = fixed_layer_height_profile(&parameters(below, 0.0, 0.1, top));
    assert_eq!(compressed, vec![0.0, below, top, 0.0]);
    let remaining_from_compressed_z = fixed_layer_height_profile(&parameters(below, 0.2, 0.1, top));
    assert_eq!(
        remaining_from_compressed_z,
        vec![0.0, below, 0.0, 0.2, top, 0.2]
    );
    assert_eq!(exact[exact.len() - 2].to_bits(), top.to_bits());
    assert_eq!(compressed[compressed.len() - 2].to_bits(), top.to_bits());
}

#[test]
fn task22a_fixed_first_pair_is_unconditional_at_and_above_top() {
    let below = f64::from_bits(0.0001_f64.to_bits() - 1);
    for (first, top, regular, expected_profile) in [
        (0.2, 0.2, 0.15, [0.0, 0.2, 0.2, 0.2]),
        (0.3, 0.2, 0.15, [0.0, 0.3, 0.3, 0.3]),
        (below, below, 0.2, [0.0, below, 0.0, 0.2]),
        (below, below / 2.0, 0.2, [0.0, below, 0.0, 0.2]),
    ] {
        let parameters = parameters(first, regular, 0.1, top);
        let profile = fixed_layer_height_profile(&parameters);
        assert_eq!(profile, expected_profile);
        let pairs =
            generate_layer_pairs(&parameters, &profile, &mut LayerBudget::default()).unwrap();
        assert_eq!(pairs, vec![LayerPair { lo: 0.0, hi: first }]);
        assert_eq!(
            planned_layers(&pairs).unwrap(),
            vec![PlannedLayer {
                id: 0,
                height: first,
                print_z: first,
                slice_z: 0.5 * first,
            }]
        );
    }
}

#[test]
fn task22a_midpoint_equal_to_top_stops_before_candidate() {
    let parameters = parameters(0.2, 0.2, 0.1, 0.3);
    let profile = fixed_layer_height_profile(&parameters);
    assert_eq!(
        generate_layer_pairs(&parameters, &profile, &mut LayerBudget::default()).unwrap(),
        vec![LayerPair { lo: 0.0, hi: 0.2 }]
    );
}

#[test]
fn task22a_layer_series_preserves_nondivisible_unaligned_top_and_records() {
    let nondivisible = parameters(0.15, 0.2, 0.1, 0.7);
    let profile = fixed_layer_height_profile(&nondivisible);
    let pairs = generate_layer_pairs(&nondivisible, &profile, &mut LayerBudget::default()).unwrap();
    let z1 = 0.15;
    let z2 = z1 + 0.2;
    let z3 = z2 + 0.2;
    let z4 = z3 + 0.2;
    let expected_pairs = vec![
        LayerPair { lo: 0.0, hi: z1 },
        LayerPair { lo: z1, hi: z2 },
        LayerPair { lo: z2, hi: z3 },
        LayerPair { lo: z3, hi: z4 },
    ];
    assert_eq!(pairs, expected_pairs);
    assert_eq!(planned_layers(&pairs).unwrap(), expected_layers(&pairs));
    assert_eq!(pairs.last().unwrap().hi, 0.75);
    let first = 0.2;
    let minimum = 0.2;
    let top = 0.8;
    let boundary = first + 0.5 * minimum;
    let mut interpolated = parameters(first, 0.2, minimum, top);
    interpolated.max_layer_height = 0.4;
    let profile = vec![0.0, 0.2, boundary, 0.2, boundary, 0.4, top, 0.2];
    let second_top = first + 0.4;
    let probe = second_top + 0.5 * minimum;
    let position = (probe - boundary) / (top - boundary);
    let interpolated_height = (1.0 - position) * 0.4 + position * 0.2;
    assert_eq!(
        generate_layer_pairs(&interpolated, &profile, &mut LayerBudget::default()).unwrap(),
        vec![
            LayerPair { lo: 0.0, hi: first },
            LayerPair {
                lo: first,
                hi: second_top
            },
            LayerPair {
                lo: second_top,
                hi: second_top + interpolated_height,
            },
        ]
    );
}

#[test]
fn task22a_layer_series_is_deterministic() {
    let parameters = parameters(0.15, 0.2, 0.1, 0.7);
    let profile = fixed_layer_height_profile(&parameters);
    let run = || {
        let pairs =
            generate_layer_pairs(&parameters, &profile, &mut LayerBudget::default()).unwrap();
        (pairs.clone(), planned_layers(&pairs).unwrap())
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(
        layer_bits(&first.1),
        vec![
            [0x3fc3333333333333, 0x3fc3333333333333, 0x3fb3333333333333],
            [0x3fc9999999999999, 0x3fd6666666666666, 0x3fd0000000000000],
            [0x3fc999999999999c, 0x3fe199999999999a, 0x3fdccccccccccccd],
            [0x3fc9999999999998, 0x3fe8000000000000, 0x3fe4cccccccccccd],
        ]
    );
}

#[test]
fn task22a_smallest_positive_regular_height_rejects_nonprogress() {
    let smallest = f64::from_bits(1);
    let parameters = parameters(0.2, smallest, smallest, 1.0);
    let profile = fixed_layer_height_profile(&parameters);
    assert_invalid(
        generate_layer_pairs(&parameters, &profile, &mut LayerBudget::default()),
        PROGRESS_ERROR,
    );
}

#[test]
fn task22a_layer_budget_allows_exact_limit_and_rejects_next() {
    let exact = parameters(1.0, 1.0, 1.0, 100_000.0);
    let pairs = generate_layer_pairs(
        &exact,
        &fixed_layer_height_profile(&exact),
        &mut LayerBudget::default(),
    )
    .unwrap();
    assert_eq!(pairs.len(), 100_000);
    for (index, pair) in pairs.iter().enumerate() {
        assert_eq!(
            *pair,
            LayerPair {
                lo: index as f64,
                hi: index as f64 + 1.0
            }
        );
    }
    let layers = planned_layers(&pairs).unwrap();
    assert_eq!(layers, expected_layers(&pairs));

    let overflow = parameters(1.0, 1.0, 1.0, 100_001.0);
    assert_invalid(
        generate_layer_pairs(
            &overflow,
            &fixed_layer_height_profile(&overflow),
            &mut LayerBudget::default(),
        ),
        LIMIT_ERROR,
    );
}

#[test]
fn task22a_layer_budget_spans_objects_and_groups() {
    let large = parameters(1.0, 1.0, 1.0, 99_998.0);
    let one = parameters(0.5, 0.5, 0.5, 0.5);
    let mut objects = vec![
        resolved(2, object_options(), Vec::new()),
        resolved(4, object_options(), Vec::new()),
        resolved(9, object_options(), Vec::new()),
    ];
    objects[0].print_objects.clear();
    objects[2].print_objects.push(ResolvedPrintObjectConfig {
        transform: Transform3d::IDENTITY,
    });
    let calls: [Cell<usize>; 10] = std::array::from_fn(|_| Cell::new(0));
    let plans = plan_resolved_objects(&objects, |_, object| {
        let call = &calls[object.source_object_index];
        call.set(call.get() + 1);
        match object.source_object_index {
            2 => Err(SliceError::UnsupportedProjectFeature("skipped".into())),
            4 => Ok(large.clone()),
            _ => Ok(one.clone()),
        }
    })
    .unwrap();
    let identities: Vec<_> = plans
        .iter()
        .map(|plan| (plan.source_object_index, plan.transform_index))
        .collect();
    assert_eq!(identities, [(4, 0), (9, 0), (9, 1)]);
    let large_layers = &plans[0].layers;
    assert_eq!(large_layers.len(), 99_998);
    assert_eq!(large_layers[0].id, 0);
    assert_eq!(large_layers[99_997].id, 99_997);
    let one_layer = PlannedLayer {
        id: 0,
        height: 0.5,
        print_z: 0.5,
        slice_z: 0.25,
    };
    assert_eq!(plans[1].layers, vec![one_layer]);
    assert_eq!(plans[2].layers, plans[1].layers);
    assert_eq!((calls[2].get(), calls[4].get(), calls[9].get()), (0, 1, 1));

    objects[2].print_objects.push(ResolvedPrintObjectConfig {
        transform: Transform3d::IDENTITY,
    });
    objects.push(resolved(12, object_options(), Vec::new()));
    assert_invalid(
        plan_resolved_objects(&objects, |_, object| match object.source_object_index {
            2 => Err(SliceError::UnsupportedProjectFeature("skipped".into())),
            4 => Ok(large.clone()),
            12 => Err(SliceError::UnsupportedProjectFeature("later".into())),
            _ => Ok(one.clone()),
        }),
        LIMIT_ERROR,
    );
}

#[test]
fn task22a_layer_generation_error_precedence_is_fixed() {
    let initial_nonfinite = parameters(f64::MAX, f64::MAX, f64::MAX, f64::MAX);
    assert_invalid(
        generate_layer_pairs(
            &initial_nonfinite,
            &fixed_layer_height_profile(&initial_nonfinite),
            &mut LayerBudget::default(),
        ),
        "nonfinite layer generation value",
    );

    let candidate_nonfinite = parameters(f64::MAX * 0.75, f64::MAX, 1.0, f64::MAX);
    assert_invalid(
        generate_layer_pairs(
            &candidate_nonfinite,
            &fixed_layer_height_profile(&candidate_nonfinite),
            &mut LayerBudget { used: 99_999 },
        ),
        "nonfinite layer generation value",
    );

    let regular = parameters(0.2, 0.2, 0.1, 1.0);
    let mut midpoint_budget = LayerBudget { used: 99_999 };
    let midpoint = parameters(0.2, 0.2, 0.1, 0.3);
    assert_eq!(
        generate_layer_pairs(
            &midpoint,
            &fixed_layer_height_profile(&midpoint),
            &mut midpoint_budget,
        )
        .unwrap(),
        vec![LayerPair { lo: 0.0, hi: 0.2 }]
    );

    let smallest = f64::from_bits(1);
    let stalled = parameters(0.2, smallest, smallest, 1.0);
    assert_invalid(
        generate_layer_pairs(
            &stalled,
            &fixed_layer_height_profile(&stalled),
            &mut LayerBudget { used: 99_999 },
        ),
        PROGRESS_ERROR,
    );
    assert_invalid(
        generate_layer_pairs(
            &regular,
            &fixed_layer_height_profile(&regular),
            &mut LayerBudget { used: 99_999 },
        ),
        LIMIT_ERROR,
    );
    assert_eq!(
        generate_layer_pairs(
            &parameters(0.2, 0.2, 0.1, 0.4),
            &[0.0, 0.2, 0.4, 0.2],
            &mut LayerBudget::default(),
        )
        .unwrap(),
        vec![
            LayerPair { lo: 0.0, hi: 0.2 },
            LayerPair { lo: 0.2, hi: 0.4 }
        ]
    );
    assert_invalid(
        planned_layers(&[LayerPair {
            lo: f64::MAX * 0.75,
            hi: f64::MAX,
        }]),
        "nonfinite layer generation value",
    );
    assert_invalid(planned_layers(&[]), "object layer pair series is empty");
}

fn parameters(first: f64, regular: f64, minimum: f64, top: f64) -> SlicingParameters {
    SlicingParameters {
        base_raft_layers: 0,
        interface_raft_layers: 0,
        base_raft_layer_height: 0.0,
        interface_raft_layer_height: 0.0,
        contact_raft_layer_height: 0.0,
        layer_height: regular,
        min_layer_height: minimum,
        max_layer_height: regular.max(minimum),
        first_print_layer_height: first,
        first_object_layer_height: first,
        first_object_layer_bridging: false,
        gap_raft_object: 0.0,
        gap_object_support: 0.0,
        gap_support_object: 0.0,
        raft_base_top_z: 0.0,
        raft_interface_top_z: 0.0,
        raft_contact_top_z: 0.0,
        object_print_z_min: 0.0,
        object_print_z_max: top,
        object_print_z_uncompensated_max: top,
        object_shrinkage_compensation_z: 1.0,
    }
}

fn expected_layers(pairs: &[LayerPair]) -> Vec<PlannedLayer> {
    pairs
        .iter()
        .enumerate()
        .map(|(id, pair)| PlannedLayer {
            id,
            height: pair.hi - pair.lo,
            print_z: pair.hi,
            slice_z: 0.5 * (pair.lo + pair.hi),
        })
        .collect()
}

fn layer_bits(layers: &[PlannedLayer]) -> Vec<[u64; 3]> {
    layers
        .iter()
        .map(|layer| {
            [
                layer.height.to_bits(),
                layer.print_z.to_bits(),
                layer.slice_z.to_bits(),
            ]
        })
        .collect()
}

fn assert_invalid<T: Debug>(result: Result<T, SliceError>, expected: &str) {
    let SliceError::InvalidInput(message) = result.unwrap_err() else {
        panic!("expected InvalidInput");
    };
    assert_eq!(message, expected);
}
