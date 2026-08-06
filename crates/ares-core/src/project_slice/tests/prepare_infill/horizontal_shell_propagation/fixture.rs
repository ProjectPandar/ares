use crate::{
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{horizontal_shell_promotion, horizontal_shell_propagation},
        region_slices::{RegionSurface, RegionSurfaceKind},
        tests::support::KsrArchive,
    },
};

pub(super) fn prepare_o25(
    bytes: impl AsRef<[u8]>,
) -> horizontal_shell_promotion::PreparedPostHorizontalShellPromotion {
    super::super::horizontal_shell_promotion::fixture::prepare(bytes)
}

pub(super) fn prepare(
    bytes: impl AsRef<[u8]>,
) -> horizontal_shell_propagation::PreparedPostHorizontalShellPropagation {
    horizontal_shell_propagation::prepare(prepare_o25(bytes)).unwrap()
}

pub(super) fn controlled(
    serial: bool,
) -> horizontal_shell_promotion::PreparedPostHorizontalShellPromotion {
    let mut archive = KsrArchive::new();
    archive.replace_unique(
        "Metadata/project_settings.config",
        "\"ensure_vertical_shell_thickness\": \"ensure_all\"",
        "\"ensure_vertical_shell_thickness\": \"ensure_moderate\"",
    );
    let mut prepared = prepare_o25(archive.bytes());
    for record in prepared.objects[0].records.iter_mut().flatten() {
        record.slices.clear();
        record.fill_surfaces.clear();
    }
    prepared.objects[0].records[0]
        .as_mut()
        .unwrap()
        .slices
        .push(RegionSurface::new(
            RegionSurfaceKind::Bottom,
            square(0, 100_000),
        ));
    let neighbor = prepared.objects[0].records[1].as_mut().unwrap();
    neighbor.fill_surfaces.push(RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        if serial {
            square(0, 100_000)
        } else {
            clipper_square(100_000)
        },
    ));
    if serial {
        neighbor.fill_surfaces.push(RegionSurface::new(
            RegionSurfaceKind::Top,
            square(50_000, 100_000),
        ));
    }
    prepared
}

fn square(x: i64, size: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, 0),
            Point::new(x + size, 0),
            Point::new(x + size, size),
            Point::new(x, size),
        ]),
        Vec::new(),
    )
}

fn clipper_square(size: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(size, size),
            Point::new(0, size),
            Point::new(0, 0),
            Point::new(size, 0),
        ]),
        Vec::new(),
    )
}
