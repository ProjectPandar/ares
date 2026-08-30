use crate::{
    ExtrusionRole, ProcessInfillPattern,
    project_slice::{
        fill_entities::{FillExtrusionEntity, generate_layer},
        prepare_infill::combine_infill,
        region_slices::RegionSurfaceKind,
        tests::prepare_infill::group_fills::focused::fixture::{
            external, graph, options_mut, record_mut, rectangle, surface,
        },
    },
};

const LAYER: usize = 1;

#[test]
fn plane_path_patterns_emit_top_bottom_and_internal_solid_entities() {
    for pattern in [
        ProcessInfillPattern::HilbertCurve,
        ProcessInfillPattern::ArchimedeanChords,
        ProcessInfillPattern::OctagramSpiral,
    ] {
        for (kind, role) in [
            (RegionSurfaceKind::Top, ExtrusionRole::TopSolidInfill),
            (RegionSurfaceKind::Bottom, ExtrusionRole::BottomSurface),
            (RegionSurfaceKind::InternalSolid, ExtrusionRole::SolidInfill),
        ] {
            let mut graph = graph();
            record_mut(&mut graph, LAYER)
                .fill_no_overlap_expolygons
                .clear();
            record_mut(&mut graph, LAYER).fill_surfaces =
                vec![surface(kind, rectangle(0, 0, 12_000_000, 8_000_000), 0)];
            let options = options_mut(&mut graph, LAYER);
            match kind {
                RegionSurfaceKind::Top => options.top_surface_pattern = pattern,
                RegionSurfaceKind::Bottom => options.bottom_surface_pattern = pattern,
                RegionSurfaceKind::InternalSolid => {
                    options.internal_solid_infill_pattern = pattern;
                }
                _ => unreachable!(),
            }

            let first = generate_layer(external(&graph), 0, LAYER).unwrap();
            let second = generate_layer(external(&graph), 0, LAYER).unwrap();

            assert_eq!(first, second, "{pattern:?} {kind:?}");
            assert_eq!(first.collections.len(), 1, "{pattern:?} {kind:?}");
            assert!(!first.collections[0].no_sort, "{pattern:?} {kind:?}");
            assert!(!first.collections[0].entities.is_empty());
            assert!(first.collections[0].entities.iter().all(|entity| {
                matches!(
                    entity,
                    FillExtrusionEntity::Path(path)
                        if path.role == role && path.polyline.is_valid()
                )
            }));
            combine_infill::dispose(graph);
        }
    }
}
