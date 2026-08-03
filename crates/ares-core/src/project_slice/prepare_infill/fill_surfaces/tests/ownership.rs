use crate::{
    ProjectSettings,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::fill_surfaces::prepare_record,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

fn square(x: i64, inset: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(x + inset, inset),
        Point::new(x + 20 - inset, inset),
        Point::new(x + 20 - inset, 20 - inset),
        Point::new(x + inset, 20 - inset),
    ])
}

fn surface(kind: RegionSurfaceKind, x: i64, index: u16) -> RegionSurface {
    RegionSurface::internal_with_metadata(
        ExPolygon::new(square(x, 0), vec![square(x, 5)]),
        f64::from(index) + 0.25,
        index + 1,
        f64::from(index) + 0.5,
        index + 2,
    )
    .clone_with_kind(kind)
}

fn allocation_snapshot(surfaces: &[RegionSurface]) -> Vec<usize> {
    let mut output = vec![surfaces.as_ptr() as usize, surfaces.len()];
    for surface in surfaces {
        let expolygon = surface.as_parts().1;
        output.extend([
            expolygon.contour().points().as_ptr() as usize,
            expolygon.holes().as_ptr() as usize,
            expolygon.holes().len(),
        ]);
        output.extend(
            expolygon
                .holes()
                .iter()
                .map(|hole| hole.points().as_ptr() as usize),
        );
    }
    output
}

#[test]
fn task22o18_retags_in_place_without_touching_geometry_order_or_metadata() {
    let mut surfaces = vec![
        surface(RegionSurfaceKind::Top, 0, 1),
        surface(RegionSurfaceKind::BottomBridge, 30, 2),
        surface(RegionSurfaceKind::Internal, 60, 3),
    ];
    let capacity = surfaces.capacity();
    let allocations = allocation_snapshot(&surfaces);
    let values = surfaces
        .iter()
        .map(|surface| {
            let (_, expolygon, thickness, layers, angle, extra) = surface.as_parts();
            (
                expolygon.clone(),
                thickness.to_bits(),
                layers,
                angle.to_bits(),
                extra,
            )
        })
        .collect::<Vec<_>>();

    let mut options = crate::RegionOptions::from_base(&ProjectSettings::default().process.region);
    options.top_shell_layers.0 = 0;
    options.bottom_shell_layers.0 = 0;
    options.sparse_infill_density.0 = 100.0;
    prepare_record(&mut surfaces, &options, false);

    assert_eq!(surfaces.capacity(), capacity);
    assert_eq!(allocation_snapshot(&surfaces), allocations);
    assert_eq!(
        surfaces
            .iter()
            .map(|surface| {
                let (_, expolygon, thickness, layers, angle, extra) = surface.as_parts();
                (
                    expolygon.clone(),
                    thickness.to_bits(),
                    layers,
                    angle.to_bits(),
                    extra,
                )
            })
            .collect::<Vec<_>>(),
        values
    );
    assert!(
        surfaces
            .iter()
            .all(|surface| surface.as_parts().0 == RegionSurfaceKind::InternalSolid)
    );
}
