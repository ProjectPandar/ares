use crate::{
    SliceError,
    geometry::{CoordinateScale, ExPolygon, Point, Polygon},
    project_slice::{
        prepare_infill::{external_surfaces, horizontal_shell_propagation},
        region_slices::{RegionSurface, RegionSurfaceKind},
        tests::support::KsrArchive,
    },
};

fn square(min: i64, max: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(min, min),
            Point::new(max, min),
            Point::new(max, max),
            Point::new(min, max),
        ]),
        Vec::new(),
    )
}

#[test]
fn task22o42_ksr_lifecycle_rebuilds_all_present_records() {
    let mut predecessor =
        super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    let input_slots = predecessor
        .objects
        .iter()
        .map(|object| {
            object
                .records
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let mut injected = 0_i64;
    for record in predecessor
        .objects
        .iter_mut()
        .flat_map(|object| &mut object.records)
        .flatten()
    {
        record.fill_surfaces.push(RegionSurface::new(
            RegionSurfaceKind::InternalVoid,
            square(injected * 10, injected * 10 + 1),
        ));
        injected += 1;
    }
    assert_eq!(injected, 460);
    let output = external_surfaces::prepare(predecessor).unwrap();

    assert_eq!(input_slots.len(), 1);
    assert_eq!(input_slots[0].len(), 460);
    assert_eq!(
        output
            .predecessor
            .objects
            .iter()
            .map(|object| object
                .records
                .iter()
                .map(Option::is_some)
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        input_slots
    );
    assert!(
        output
            .predecessor
            .objects
            .iter()
            .flat_map(|object| &object.records)
            .flatten()
            .flat_map(|record| &record.fill_surfaces)
            .all(|surface| surface.as_parts().0 != RegionSurfaceKind::InternalVoid)
    );

    external_surfaces::dispose(output);
}

#[test]
fn task22o42_ksr_adapter_reads_composed_options_and_scaled_prelude_values() {
    let predecessor =
        super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(predecessor.predecessor.scale, CoordinateScale::Normal);
    assert!(
        !predecessor
            .predecessor
            .resolved
            .views
            .full
            .process
            .print
            .spiral_mode
            .0
    );

    let traversal = &predecessor.predecessor.objects[0];
    let prelude = &traversal.predecessor.predecessor.predecessor.predecessor;
    let (_, inputs) = prelude.object.as_parts();
    let first_input = inputs[0].as_ref().unwrap();
    let first_options = prelude.object.region_options(first_input);
    assert_eq!(
        (
            first_options.wall_loops.0,
            first_options.bridge_angle.0,
            first_options.relative_bridge_angle.0,
            first_options.sparse_infill_density.0,
            first_options.minimum_sparse_infill_area.0,
        ),
        (2, 0.0, false, 15.0, 15.0)
    );
    assert_eq!(first_input.model_rotation_rad.to_bits(), 0);

    let first = prelude.records[0].as_ref().unwrap();
    let later = prelude.records[1].as_ref().unwrap();
    assert_eq!(
        (
            first.perimeter_width,
            first.perimeter_spacing,
            first.external_width,
            first.external_spacing,
            first.solid_infill_spacing,
        ),
        (500_000, 457_079, 500_000, 457_079, 457_079)
    );
    assert_eq!(
        (
            later.perimeter_width,
            later.perimeter_spacing,
            later.external_width,
            later.external_spacing,
            later.solid_infill_spacing,
        ),
        (449_999, 407_079, 419_999, 377_079, 377_079)
    );

    horizontal_shell_propagation::dispose(predecessor);

    assert_eq!(
        controlled_sparse_kind(KsrArchive::new()),
        RegionSurfaceKind::InternalSolid
    );
    let mut no_promotion = KsrArchive::new();
    no_promotion.replace_unique(
        "Metadata/project_settings.config",
        "\"minimum_sparse_infill_area\": \"15\"",
        "\"minimum_sparse_infill_area\": \"0\"",
    );
    assert_eq!(
        controlled_sparse_kind(no_promotion),
        RegionSurfaceKind::Internal
    );
}

fn controlled_sparse_kind(archive: KsrArchive) -> RegionSurfaceKind {
    let mut predecessor = super::horizontal_shell_propagation::fixture::prepare(archive.bytes());
    let record = predecessor.objects[0].records[0].as_mut().unwrap();
    record.fill_surfaces = vec![RegionSurface::new(
        RegionSurfaceKind::Internal,
        square(0, 1_000_000),
    )];

    let output = external_surfaces::prepare(predecessor).unwrap();
    let surfaces = &output.predecessor.objects[0].records[0]
        .as_ref()
        .unwrap()
        .fill_surfaces;
    assert_eq!(surfaces.len(), 1);
    let kind = surfaces[0].as_parts().0;
    external_surfaces::dispose(output);
    kind
}

#[test]
fn task22o42_stage_maps_geometry_error_and_disposes_owned_predecessor() {
    horizontal_shell_propagation::reset_hooks();
    let mut predecessor =
        super::horizontal_shell_propagation::fixture::prepare(KsrArchive::new().bytes());
    assert_eq!(horizontal_shell_propagation::invocations(), 1);
    assert_eq!(horizontal_shell_propagation::disposals(), 0);
    let record = predecessor.objects[0].records[0].as_mut().unwrap();
    record.fill_surfaces.clear();
    record.fill_surfaces.push(RegionSurface::new(
        RegionSurfaceKind::InternalSolid,
        ExPolygon::new(
            Polygon::new(vec![
                Point::new(0x4000_0000_0000_0000, 0),
                Point::new(0x4000_0000_0000_0000, 10),
                Point::new(0x3fff_ffff_ffff_ffff, 10),
            ]),
            Vec::new(),
        ),
    ));

    let result = external_surfaces::prepare(predecessor);
    let Err(SliceError::InvalidInput(message)) = result else {
        panic!("O42 range failure must map to invalid project input")
    };
    assert_eq!(
        message,
        "external-surface polygon coordinate is outside the supported Clipper range"
    );
    assert_eq!(horizontal_shell_propagation::disposals(), 1);
    horizontal_shell_propagation::reset_hooks();
}
