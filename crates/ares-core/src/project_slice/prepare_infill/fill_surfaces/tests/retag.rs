use crate::{
    ProjectSettings,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::fill_surfaces::prepare_record,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn rectangle(x: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + 10, 0),
            Point::new(x + 10, 10),
            Point::new(x, 10),
        ]),
        Vec::new(),
    )
}

fn surfaces() -> Vec<RegionSurface> {
    [
        RegionSurfaceKind::Top,
        RegionSurfaceKind::Bottom,
        RegionSurfaceKind::BottomBridge,
        RegionSurfaceKind::Internal,
        RegionSurfaceKind::InternalSolid,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, kind)| RegionSurface::new(kind, rectangle(index as i64 * 20)))
    .collect()
}

fn kinds(surfaces: &[RegionSurface]) -> Vec<RegionSurfaceKind> {
    surfaces
        .iter()
        .map(|surface| surface.as_parts().0)
        .collect()
}

fn options(top: i32, bottom: i32, density: f64) -> crate::RegionOptions {
    let mut options = crate::RegionOptions::from_base(&ProjectSettings::default().process.region);
    options.top_shell_layers.0 = top;
    options.bottom_shell_layers.0 = bottom;
    options.sparse_infill_density.0 = density;
    options
}

#[test]
fn task22o18_three_passes_are_sequential_and_source_literal() {
    let mut value = surfaces();
    prepare_record(&mut value, &options(0, 0, 100.0), false);
    assert_eq!(
        kinds(&value),
        [
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
            RegionSurfaceKind::InternalSolid,
        ]
    );

    let mut value = surfaces();
    prepare_record(&mut value, &options(0, 0, 15.0), false);
    assert_eq!(
        kinds(&value),
        [
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::InternalSolid,
        ]
    );
}

#[test]
fn task22o18_each_pass_changes_only_its_literal_source_kinds() {
    for (options, expected) in [
        (
            options(0, 1, 15.0),
            [
                RegionSurfaceKind::Internal,
                RegionSurfaceKind::Bottom,
                RegionSurfaceKind::BottomBridge,
                RegionSurfaceKind::Internal,
                RegionSurfaceKind::InternalSolid,
            ],
        ),
        (
            options(1, 0, 15.0),
            [
                RegionSurfaceKind::Top,
                RegionSurfaceKind::Internal,
                RegionSurfaceKind::Internal,
                RegionSurfaceKind::Internal,
                RegionSurfaceKind::InternalSolid,
            ],
        ),
        (
            options(1, 1, 100.0),
            [
                RegionSurfaceKind::Top,
                RegionSurfaceKind::Bottom,
                RegionSurfaceKind::BottomBridge,
                RegionSurfaceKind::InternalSolid,
                RegionSurfaceKind::InternalSolid,
            ],
        ),
    ] {
        let mut value = surfaces();
        prepare_record(&mut value, &options, false);
        assert_eq!(kinds(&value), expected);
    }
}

#[test]
fn task22o18_density_comparison_is_strict_and_uses_source_epsilon() {
    for (density, solid) in [(99.99995, true), (100.0 - 0.0001, false), (99.9998, false)] {
        let mut value = vec![RegionSurface::new(
            RegionSurfaceKind::Internal,
            rectangle(0),
        )];
        prepare_record(&mut value, &options(1, 1, density), false);
        assert_eq!(
            value[0].as_parts().0 == RegionSurfaceKind::InternalSolid,
            solid,
            "density {density}"
        );
    }
}

#[test]
fn task22o18_spiral_guards_skip_first_and_third_but_not_second() {
    let mut value = surfaces();
    prepare_record(&mut value, &options(0, 0, 100.0), true);
    assert_eq!(
        kinds(&value),
        [
            RegionSurfaceKind::Top,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::Internal,
            RegionSurfaceKind::InternalSolid,
        ]
    );
}
