use crate::project_slice::perimeters::prepare_post_classic_prelude;

use super::super::super::support::ksr_project;

#[test]
fn task22o1_prelude_consumes_task22n_flows_and_builds_aligned_geometry() {
    let prepared = prepare_post_classic_prelude(ksr_project()).unwrap();
    assert!(!prepared.objects.is_empty());
    let mut populated = 0;
    let mut lower_series = 0;
    for object in prepared.objects {
        let (before, records) = object.into_parts();
        let (_, inputs) = before.as_parts();
        assert_eq!(inputs.len(), records.len());
        for (input, prelude) in inputs.iter().zip(&records) {
            assert_eq!(input.is_some(), prelude.is_some());
            let (Some(input), Some(prelude)) = (input, prelude) else {
                continue;
            };

            populated += 1;
            assert_eq!(
                prelude.perimeter_width,
                (f64::from(input.perimeter_flow.width) / prepared.scale.factor()) as i64
            );
            assert_eq!(
                prelude.external_spacing,
                (f64::from(input.ext_perimeter_flow.spacing) / prepared.scale.factor()) as i64
            );
            assert_eq!(
                prelude.smaller_external_flow.height.to_bits(),
                input.ext_perimeter_flow.height.to_bits()
            );
            assert_eq!(
                prelude.lower_polygons_series.len(),
                input.lower_layer_index.map_or(0, |_| 2)
            );
            lower_series += usize::from(!prelude.lower_polygons_series.is_empty());
            assert!(!prelude.surfaces.is_empty());
            assert!(
                prelude
                    .surfaces
                    .iter()
                    .all(|surface| !surface.polygons.is_empty())
            );
        }
    }
    assert!(populated > 0);
    assert!(lower_series > 0);
}

#[test]
fn task22o1_prelude_preserves_flow_and_scaled_float_conversion_boundaries() {
    let prepared = prepare_post_classic_prelude(ksr_project()).unwrap();
    let records = prepared
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .collect::<Vec<_>>();

    assert_eq!(records[1].external_width, 419_999);
    assert_eq!(records[1].external_to_internal_spacing, 435_000);
    assert_eq!(
        records[1].smaller_external_flow.width.to_bits(),
        0x3ec1_cd6a
    );
    assert_eq!(
        records[1].smaller_external_flow.spacing.to_bits(),
        0x3eab_d3c2
    );
    assert_eq!(
        records[1].smaller_external_flow.mm3_per_mm.to_bits(),
        0x3fb1_2ec6_8000_0000
    );
}

#[test]
fn task22o1_prelude_is_repeatable_without_fixture_identity_branches() {
    type FlowSummary = (i64, i64, u32, u64);
    type ObjectSummary = (usize, usize, Vec<FlowSummary>);

    fn summary() -> Vec<ObjectSummary> {
        prepare_post_classic_prelude(ksr_project())
            .unwrap()
            .objects
            .into_iter()
            .map(|object| {
                let (before, records) = object.into_parts();
                let identity = before.identity();
                let records = records
                    .into_iter()
                    .flatten()
                    .map(|record| {
                        (
                            record.external_to_internal_spacing,
                            record.smaller_external_minimum_spacing,
                            record.smaller_external_flow.width.to_bits(),
                            record.smaller_external_flow.mm3_per_mm.to_bits(),
                        )
                    })
                    .collect();
                (identity.0, identity.1, records)
            })
            .collect()
    }
    assert_eq!(summary(), summary());
}

#[test]
fn task22o1_prelude_uses_arc_resolution_and_top_wall_count_from_typed_options() {
    let prepared = prepare_post_classic_prelude(ksr_project()).unwrap();
    let records = prepared
        .objects
        .into_iter()
        .flat_map(|object| object.into_parts().1)
        .flatten()
        .collect::<Vec<_>>();
    assert!(
        records
            .iter()
            .all(|record| record.surface_simplify_resolution > 0.0)
    );
    let top = records.last().unwrap();
    assert!(top.surfaces.iter().all(|surface| surface.loop_number == 0));
}
