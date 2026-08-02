use crate::project_slice::perimeters::{
    classic::materialize, prepare_post_classic_raw_paths, prepare_post_classic_traversal,
};

use super::super::super::super::support::ksr_project;

#[test]
fn task22o7_overhang_materialization_uses_only_the_final_lower_series_element() {
    let baseline = prepare_post_classic_raw_paths(ksr_project()).unwrap();
    let mut traversal = prepare_post_classic_traversal(ksr_project()).unwrap();
    let mut changed = 0;
    for object in &mut traversal.objects {
        let prelude_records = &mut object
            .predecessor
            .predecessor
            .predecessor
            .predecessor
            .records;
        for record in prelude_records.iter_mut().flatten() {
            changed += clear_first(&mut record.lower_polygons_series);
            changed += clear_first(&mut record.external_lower_polygons_series);
            changed += clear_first(&mut record.smaller_external_lower_polygons_series);
        }
    }
    assert!(changed > 0);

    let with_changed_earlier_series = materialize::finish(traversal).unwrap();
    assert_eq!(baseline.objects, with_changed_earlier_series.objects);
}

fn clear_first(series: &mut [Vec<crate::geometry::Polygon>]) -> usize {
    if series.len() <= 1 {
        return 0;
    }
    series[0].clear();
    1
}
