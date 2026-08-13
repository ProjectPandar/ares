use super::*;

#[test]
fn task22o67_real_two_step_geometry_is_repeatable_and_resets_metadata() {
    let rich = RegionSurface::internal_with_metadata(
        expolygon(rectangle(0, 0, 200, 100), vec![rectangle(80, 20, 120, 80)]),
        0.8,
        4,
        1.25,
        7,
    );
    let other = surface(
        RegionSurfaceKind::InternalVoid,
        expolygon(rectangle(300, 0, 400, 100), Vec::new()),
    );
    let surfaces = vec![rich, other];
    let cut = vec![rectangle(0, 0, 50, 100)];
    let ensuring = vec![expolygon(rectangle(150, 0, 200, 100), Vec::new())];
    let before = surface_snapshot(&surfaces);
    let cut_before = cut.clone();
    let ensuring_before = snapshot_ex(&ensuring);
    let source_ptr = surfaces[0].as_parts().1.contour().points().as_ptr();

    let first = rebuild_internal_infills(&surfaces, &cut, &ensuring).unwrap();
    let second = rebuild_internal_infills(&surfaces, &cut, &ensuring).unwrap();
    assert_eq!(surface_snapshot(&first), surface_snapshot(&second));
    assert_eq!(first.len(), 1);
    assert_eq!(
        snapshot_ex(
            &first
                .iter()
                .map(|surface| surface.as_parts().1.clone())
                .collect::<Vec<_>>()
        ),
        vec![(vec![(150, 100), (50, 100), (50, 0), (150, 0)], Vec::new(),)]
    );
    let (kind, _, thickness, layers, angle, extra) = first[0].as_parts();
    assert_eq!(kind, RegionSurfaceKind::Internal);
    assert_eq!((thickness, layers, angle, extra), (-1.0, 1, -1.0, 0));
    assert_ne!(
        first[0].as_parts().1.contour().points().as_ptr(),
        source_ptr
    );
    assert_eq!(surface_snapshot(&surfaces), before);
    assert_eq!(cut, cut_before);
    assert_eq!(snapshot_ex(&ensuring), ensuring_before);
    assert_eq!(
        surfaces[0].as_parts().1.contour().points().as_ptr(),
        source_ptr
    );
}

#[test]
fn task22o67_natural_first_subject_first_clip_and_second_clip_errors_are_exact() {
    let high = 0x3fff_ffff_ffff_ffff_i64;
    let invalid_surface = [surface(
        RegionSurfaceKind::Internal,
        expolygon(rectangle(high + 1, 0, high + 100, 100), Vec::new()),
    )];
    assert_eq!(
        rebuild_internal_infills(&invalid_surface, &[], &[])
            .err()
            .unwrap(),
        ClipperError::CoordinateOutOfRange
    );

    let valid = [surface(
        RegionSurfaceKind::Internal,
        expolygon(rectangle(0, 0, 100, 100), Vec::new()),
    )];
    let invalid_cut = [rectangle(high + 1, 0, high + 100, 100)];
    assert_eq!(
        rebuild_internal_infills(&valid, &invalid_cut, &[])
            .err()
            .unwrap(),
        ClipperError::CoordinateOutOfRange
    );
    let invalid_ensuring = [expolygon(
        rectangle(high + 1, 0, high + 100, 100),
        Vec::new(),
    )];
    assert_eq!(
        rebuild_internal_infills(&valid, &[], &invalid_ensuring)
            .err()
            .unwrap(),
        ClipperError::CoordinateOutOfRange
    );
}

#[test]
fn task22o67_complete_first_erosion_still_returns_empty_after_second_difference() {
    let surfaces = [surface(
        RegionSurfaceKind::Internal,
        expolygon(rectangle(0, 0, 100, 100), Vec::new()),
    )];
    assert!(
        rebuild_internal_infills(&surfaces, &[rectangle(-10, -10, 110, 110)], &[])
            .unwrap()
            .is_empty()
    );
}
