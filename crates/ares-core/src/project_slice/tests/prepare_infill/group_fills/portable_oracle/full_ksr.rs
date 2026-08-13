use std::collections::BTreeMap;

use crate::project_slice::{
    group_fills::{self, BaseGroupedFills, SurfaceFillPattern},
    prepare_infill::{combine_infill, external_surfaces::PreparedPostExternalSurfaces},
    tests::{prepare_infill::bridge_over_infill::transaction::snapshot, support::KsrArchive},
};

use super::*;

#[derive(Clone, Copy)]
struct LayerHeader {
    id: usize,
    height: f64,
    print_z: f64,
}

#[test]
fn task22o73_real_ksr_all_460_layers_match_portable_pre_narrow_oracle_and_repeat() {
    let (headers, first, second) = {
        let input = super::super::super::combine_infill::prepare_o71(KsrArchive::new());
        let graph = combine_infill::prepare(input).unwrap();
        let external = &graph.predecessor.predecessor;
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
        let headers = plan
            .layers
            .iter()
            .map(|layer| LayerHeader {
                id: layer.id,
                height: layer.height,
                print_z: layer.print_z,
            })
            .collect::<Vec<_>>();

        assert_eq!(headers.len(), KSR_TOTALS.layers);
        assert_eq!(horizontal.objects[0].records.len(), KSR_TOTALS.layers);
        assert_eq!(inputs.len(), KSR_TOTALS.layers);
        assert_eq!(
            headers.iter().map(|layer| layer.id).collect::<Vec<_>>(),
            (0..KSR_TOTALS.layers).collect::<Vec<_>>()
        );
        assert_eq!(
            horizontal.objects[0]
                .records
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>(),
            inputs.iter().map(Option::is_some).collect::<Vec<_>>()
        );

        let source_index = prelude.object.identity().0;
        let object = &traversal
            .resolved
            .objects
            .iter()
            .find(|object| object.source_object_index == source_index)
            .unwrap()
            .object;
        let nozzles = &traversal.resolved.views.full.project.print.nozzle_diameter;
        let before = snapshot(&graph.predecessor);
        let planned_before = plan.layers.clone();
        let object_before = object.clone();
        let nozzles_before = nozzles.clone();
        let regions_before = inputs
            .iter()
            .map(|input| {
                input
                    .as_ref()
                    .map(|input| prelude.object.region_options(input).clone())
            })
            .collect::<Vec<_>>();

        let first = group_all_layers(external, KSR_TOTALS.layers);
        let second = group_all_layers(external, KSR_TOTALS.layers);

        let after = snapshot(&graph.predecessor);
        assert_eq!(after.bytes, before.bytes);
        assert_eq!(after.bridge_layers, before.bridge_layers);
        assert_eq!(after.bridge_surfaces, before.bridge_surfaces);
        assert_eq!(
            after.bridge_expolygon_points,
            before.bridge_expolygon_points
        );
        assert_eq!(plan.layers, planned_before);
        assert_eq!(object, &object_before);
        assert_eq!(nozzles, &nozzles_before);
        assert_eq!(
            inputs
                .iter()
                .map(|input| {
                    input
                        .as_ref()
                        .map(|input| prelude.object.region_options(input).clone())
                })
                .collect::<Vec<_>>(),
            regions_before
        );

        combine_infill::dispose(graph);
        (headers, first, second)
    };

    let first = adapt_layers(&headers, &first);
    let second = adapt_layers(&headers, &second);
    let first_totals = totals(&first);
    assert_eq!(first_totals, KSR_TOTALS);
    assert_ne!(first_totals, O74_POST_TOTALS);
    assert_eq!(totals(&second), KSR_TOTALS);
    assert_distributions(&first);
    assert_empty_layer_suffix(&first);

    let first = encode(&first);
    let second = encode(&second);
    assert_eq!(second.metadata, first.metadata);
    assert_eq!(second.canonical_geometry, first.canonical_geometry);
    assert_eq!(second.layer_table, first.layer_table);
    let metadata_sha256 = sha256_hex(&first.metadata);
    let geometry_sha256 = sha256_hex(&first.canonical_geometry);
    let table_sha256 = sha256_hex(&first.layer_table);
    assert_eq!(metadata_sha256, PRE_METADATA_SHA256);
    assert_eq!(geometry_sha256, PRE_CANONICAL_GEOMETRY_SHA256);
    assert_eq!(table_sha256, PRE_LAYER_TABLE_SHA256);
    assert_ne!(metadata_sha256, O74_POST_METADATA_SHA256);
    assert_ne!(geometry_sha256, O74_POST_CANONICAL_GEOMETRY_SHA256);
    assert_ne!(table_sha256, O74_POST_LAYER_TABLE_SHA256);
}

fn group_all_layers(
    external: &PreparedPostExternalSurfaces,
    layer_count: usize,
) -> Vec<BaseGroupedFills> {
    (0..layer_count)
        .map(|layer_index| group_fills::group_fills_base(external, 0, layer_index))
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
}

fn adapt_layers<'a>(
    headers: &[LayerHeader],
    grouped: &'a [BaseGroupedFills],
) -> Vec<OracleLayer<'a>> {
    assert_eq!(headers.len(), grouped.len());
    headers
        .iter()
        .zip(grouped)
        .map(|(header, grouped)| OracleLayer {
            layer_id: header.id,
            layer_height: header.height,
            print_z: header.print_z,
            lock_counts: OracleLockCounts {
                skin_density: grouped.lock_region_param.skin_density_params.len(),
                skeleton_density: grouped.lock_region_param.skeleton_density_params.len(),
                skin_flow: grouped.lock_region_param.skin_flow_params.len(),
                skeleton_flow: grouped.lock_region_param.skeleton_flow_params.len(),
            },
            groups: grouped
                .surface_fills
                .iter()
                .map(|fill| OracleGroup {
                    region_id: fill.region_id,
                    representative: OracleRepresentative {
                        kind: fill.representative.kind as u8,
                        thickness: fill.representative.thickness,
                        thickness_layers: fill.representative.thickness_layers,
                        bridge_angle: fill.representative.bridge_angle,
                        extra_perimeters: fill.representative.extra_perimeters,
                    },
                    params: OracleParams {
                        extruder: fill.params.extruder,
                        pattern: match fill.params.pattern {
                            SurfaceFillPattern::Configured(pattern) => {
                                configured_pattern_rank(pattern)
                            }
                            SurfaceFillPattern::ConcentricInternal => 29,
                        },
                        spacing: fill.params.spacing,
                        overlap: fill.params.overlap,
                        angle: fill.params.angle,
                        fixed_angle: fill.params.fixed_angle,
                        bridge: fill.params.bridge,
                        bridge_angle: fill.params.bridge_angle,
                        density: fill.params.density,
                        multiline: fill.params.multiline,
                        anchor_length: fill.params.anchor_length,
                        anchor_length_max: fill.params.anchor_length_max,
                        flow: OracleFlow {
                            width: fill.params.flow.width,
                            height: fill.params.flow.height,
                            spacing: fill.params.flow.spacing,
                            nozzle_diameter: fill.params.flow.nozzle_diameter,
                            bridge: fill.params.flow.bridge,
                        },
                        extrusion_role: extrusion_role_rank(fill.params.extrusion_role),
                        role_speed: fill.params.role_speed,
                        lateral_lattice_angle_1: fill.params.lateral_lattice_angle_1,
                        lateral_lattice_angle_2: fill.params.lateral_lattice_angle_2,
                        infill_lock_depth: fill.params.infill_lock_depth,
                        skin_infill_depth: fill.params.skin_infill_depth,
                        symmetric_infill_y_axis: fill.params.symmetric_infill_y_axis,
                        infill_overhang_angle: fill.params.infill_overhang_angle,
                        gyroid_optimized: fill.params.gyroid_optimized,
                    },
                    region_id_group: &fill.region_id_group,
                    fills: &fill.expolygons,
                    no_overlap: &fill.no_overlap_expolygons,
                })
                .collect(),
        })
        .collect()
}

fn assert_distributions(layers: &[OracleLayer<'_>]) {
    assert_eq!(
        frequencies(layers.iter().map(|layer| layer.groups.len())),
        KSR_GROUP_HISTOGRAM.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.representative.kind)
        ),
        KSR_KIND_COUNTS.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.params.pattern)
        ),
        KSR_PATTERN_COUNTS.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.params.extrusion_role)
        ),
        KSR_ROLE_COUNTS.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.params.extruder)
        ),
        KSR_EXTRUDER_COUNTS.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.params.bridge)
        ),
        KSR_PARAMS_BRIDGE_COUNTS.to_vec()
    );
    assert_eq!(
        frequencies(
            layers
                .iter()
                .flat_map(|layer| &layer.groups)
                .map(|group| group.params.flow.bridge)
        ),
        KSR_FLOW_BRIDGE_COUNTS.to_vec()
    );

    let lock_counts = layers
        .iter()
        .fold(OracleLockCounts::default(), |counts, layer| {
            OracleLockCounts {
                skin_density: counts.skin_density + layer.lock_counts.skin_density,
                skeleton_density: counts.skeleton_density + layer.lock_counts.skeleton_density,
                skin_flow: counts.skin_flow + layer.lock_counts.skin_flow,
                skeleton_flow: counts.skeleton_flow + layer.lock_counts.skeleton_flow,
            }
        });
    assert_eq!(lock_counts, KSR_LOCK_COUNTS);
}

fn assert_empty_layer_suffix(layers: &[OracleLayer<'_>]) {
    assert_eq!(layers[41].groups.len(), 5);
    assert_eq!(layers[70].groups.len(), 8);
    assert_eq!(layers[255].groups.len(), 1);
    assert!(layers[260].groups.is_empty());
    assert_eq!(
        layers
            .iter()
            .filter(|layer| layer.groups.is_empty())
            .map(|layer| layer.layer_id)
            .collect::<Vec<_>>(),
        (260..460).collect::<Vec<_>>()
    );
}

fn frequencies<T: Copy + Ord>(values: impl IntoIterator<Item = T>) -> Vec<(T, usize)> {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0) += 1;
    }
    counts.into_iter().collect()
}
