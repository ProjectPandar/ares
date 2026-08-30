use crate::{
    OrcaBool, ProcessInfillPattern,
    project_slice::{
        fill_entities::{FillExtrusionEntity, generate_layer},
        perimeters::classic::{gap_extrusion::GapFillEntity, materialize::ExtrusionRole},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::prepare_infill::group_fills::focused::fixture::{
            external, external_mut, graph, options_mut, record_mut, rectangle, surface,
        },
    },
};

const LAYER: usize = 1;

#[test]
fn configured_concentric_materializes_standard_solid_role() {
    let mut graph = graph();
    options_mut(&mut graph, LAYER).internal_solid_infill_pattern = ProcessInfillPattern::Concentric;
    let shape = rectangle(0, 0, 8_000_000, 8_000_000);
    record_mut(&mut graph, LAYER).fill_no_overlap_expolygons = vec![shape.clone()];
    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape, 0)];

    let entities = generate_layer(external(&graph), 0, LAYER).unwrap();

    assert!(entities.collections.iter().any(|collection| {
        collection.entities.iter().any(|entity| {
            matches!(
                entity,
                FillExtrusionEntity::VariableWidth(GapFillEntity::Path(path))
                    if path.role == ExtrusionRole::SolidInfill
            )
        })
    }));
    combine_infill::dispose(graph);
}

#[test]
fn task22o92_concentric_internal_stays_one_fill_collection() {
    let mut graph = graph();
    external_mut(&mut graph)
        .predecessor
        .predecessor
        .resolved
        .objects[0]
        .object
        .detect_narrow_internal_solid_infill = OrcaBool(true);
    let shape = rectangle(0, 0, 300_000, 8_000_000);
    record_mut(&mut graph, LAYER).fill_no_overlap_expolygons = vec![shape.clone()];
    record_mut(&mut graph, LAYER).fill_surfaces =
        vec![surface(RegionSurfaceKind::InternalSolid, shape, 0)];

    let entities = generate_layer(external(&graph), 0, LAYER).unwrap();

    assert_eq!(entities.collections.len(), 1);
    assert!(entities.thin_fills.is_empty());
    combine_infill::dispose(graph);
}
