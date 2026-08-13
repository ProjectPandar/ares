use super::*;
use crate::geometry::intersection_polygons_paths;

#[test]
fn task22o57_passes_expansion_as_subject_and_deep_area_as_clip() {
    let expansion = [rectangle(0, 0, 100, 100), rectangle(0, 0, 100, 100)];
    let deep = [rectangle(0, 0, 100, 100), rectangle(0, 50, 100, 150)];
    let output = intersect_expansion_with_deep_using(&expansion, &deep, |subject, clip| {
        assert_eq!(subject.as_ptr(), expansion.as_ptr());
        assert_eq!(clip.as_ptr(), deep.as_ptr());
        assert_eq!(snapshot_polygons(subject), snapshot_polygons(&expansion));
        assert_eq!(snapshot_polygons(clip), snapshot_polygons(&deep));
        intersection_polygons_paths(subject, clip)
    })
    .unwrap();

    assert_eq!(
        snapshot_polygons(&output),
        vec![vec![(100, 100), (0, 100), (0, 0), (100, 0)]]
    );
    assert_ne!(
        snapshot_polygons(&output),
        snapshot_polygons(&intersection_polygons_paths(&deep, &expansion).unwrap())
    );
}

#[test]
fn task22o57_passes_all_lower_lines_to_one_intersection_in_source_order() {
    let lines = [
        line(&[(-50, 25), (250, 25)]),
        line(&[(75, -50), (75, 250)]),
        line(&[(-50, 250), (250, -50)]),
    ];
    let anchor_area = [rectangle(0, 0, 100, 100)];
    let output = clip_lower_lines_using(&lines, &anchor_area, |subject, clip| {
        assert_eq!(subject.as_ptr(), lines.as_ptr());
        assert_eq!(clip.as_ptr(), anchor_area.as_ptr());
        assert_eq!(snapshot_polylines(subject), snapshot_polylines(&lines));
        assert_eq!(snapshot_polygons(clip), snapshot_polygons(&anchor_area));
        Ok(subject.to_vec())
    })
    .unwrap();

    assert_eq!(snapshot_polylines(&output), snapshot_polylines(&lines));
}
