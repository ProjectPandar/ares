use crate::project_slice::perimeters::{
    classic::{gap_extrusion, gap_extrusion::GapFillEntity},
    prepare_post_classic_gap_extrusion, prepare_post_classic_medial_gap,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o14_ksr_preserves_medial_markers_and_materializes_gap_remainder() {
    let output = prepare_post_classic_gap_extrusion(ksr_project()).unwrap();
    let mut absent = 0;
    let mut present = 0;
    let mut entities = 0;
    let mut remaining = 0;
    for surface in output
        .objects
        .iter()
        .flat_map(|object| object.records.iter().flatten())
        .flat_map(|record| &record.surfaces)
    {
        match &surface.medial {
            None => absent += 1,
            Some(domain) => {
                present += 1;
                assert!(
                    domain.polylines.iter().all(|polyline| {
                        polyline.width.len() == (polyline.points.len() - 1) * 2
                    })
                );
            }
        }
        entities += surface.gap_fill.entities.len();
        remaining += surface.remaining.len();
        for entity in &surface.gap_fill.entities {
            let paths = match entity {
                GapFillEntity::Path(path) => std::slice::from_ref(path),
                GapFillEntity::Loop(paths) => paths,
            };
            assert!(paths.iter().all(|path| {
                path.polyline.points.iter().all(|point| point.z == 0)
                    && path.width.is_finite()
                    && path.height.is_finite()
                    && path.mm3_per_mm.is_finite()
            }));
        }
    }
    assert!(absent > 0);
    assert!(present > 0);
    assert!(entities > 0);
    assert!(remaining > 0);
}

#[test]
fn task22o14_some_empty_stays_present_and_keeps_onion_last_unchanged() {
    let mut source = prepare_post_classic_medial_gap(ksr_project()).unwrap();
    let (object, record, surface) = source
        .objects
        .iter()
        .enumerate()
        .find_map(|(object_index, object)| {
            object
                .records
                .iter()
                .enumerate()
                .find_map(|(record_index, record)| {
                    record.as_ref()?.surfaces.iter().enumerate().find_map(
                        |(surface_index, surface)| {
                            surface
                                .medial
                                .as_ref()
                                .map(|_| (object_index, record_index, surface_index))
                        },
                    )
                })
        })
        .unwrap();
    source.objects[object].records[record]
        .as_mut()
        .unwrap()
        .surfaces[surface]
        .medial
        .as_mut()
        .unwrap()
        .polylines
        .clear();
    let expected = source.predecessor.objects[object]
        .predecessor
        .predecessor
        .records[record]
        .as_ref()
        .unwrap()
        .surfaces[surface]
        .last
        .clone();
    let output = gap_extrusion::finish(source).unwrap();
    let surface = &output.objects[object].records[record]
        .as_ref()
        .unwrap()
        .surfaces[surface];
    assert!(surface.medial.as_ref().unwrap().polylines.is_empty());
    assert!(surface.gap_fill.entities.is_empty());
    assert_eq!(surface.remaining, expected);
}
