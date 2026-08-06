use crate::{
    ProcessRegionSourceOptions, RegionOptions,
    geometry::{ExPolygon, Point, Polygon},
    project_slice::{
        perimeters::types::{Flow, PerimeterDispatch, PerimeterInputRecord, RegionLayerIndex},
        prepare_infill::surface_type_detection::types::PreparedSurfaceTypeRecord,
        region_slices::{RegionSurface, RegionSurfaceKind},
    },
};

pub(super) fn square(x: i64, y: i64, size: i64) -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(x, y),
            Point::new(x + size, y),
            Point::new(x + size, y + size),
            Point::new(x, y + size),
        ]),
        Vec::new(),
    )
}

pub(super) fn holed_square(x: i64, y: i64, size: i64) -> ExPolygon {
    let low_x = x + size / 4;
    let low_y = y + size / 4;
    let high_x = low_x + size / 2;
    let high_y = low_y + size / 2;
    ExPolygon::new(
        square(x, y, size).contour().clone(),
        vec![Polygon::new(vec![
            Point::new(low_x, low_y),
            Point::new(low_x, high_y),
            Point::new(high_x, high_y),
            Point::new(high_x, low_y),
        ])],
    )
}

pub(super) fn surface(kind: RegionSurfaceKind, expolygon: ExPolygon) -> RegionSurface {
    RegionSurface::internal_with_metadata(expolygon, 2.5, 3, 0.75, 4).clone_with_kind(kind)
}

pub(super) fn record(
    slices: Vec<RegionSurface>,
    fill_surfaces: Vec<RegionSurface>,
) -> PreparedSurfaceTypeRecord {
    PreparedSurfaceTypeRecord {
        perimeters: Vec::new(),
        thin_fills: Vec::new(),
        slices,
        fill_surfaces,
        fill_expolygons: Vec::new(),
        fill_no_overlap_expolygons: Vec::new(),
    }
}

pub(super) fn options() -> RegionOptions {
    RegionOptions::from_base(&ProcessRegionSourceOptions::default())
}

pub(super) fn input(external_width: f32, solid_width: f32) -> PerimeterInputRecord {
    let base = flow(0.4);
    PerimeterInputRecord {
        source_object_index: 0,
        transform_index: 0,
        planned_layer_index: 0,
        layer_id: 0,
        region_id: 0,
        compatible_region_ids: [0],
        current: RegionLayerIndex {
            region_index: 0,
            layer_index: 0,
        },
        lower_layer_index: None,
        upper_layer_index: None,
        upper_same_region: None,
        layer_height: 0.2,
        slice_z: 0.1,
        perimeter_flow: base,
        ext_perimeter_flow: flow(external_width),
        overhang_flow: base,
        solid_infill_flow: flow(solid_width),
        spiral_mode: false,
        model_rotation_rad: 0.0,
        dispatch: PerimeterDispatch::Classic,
    }
}

fn flow(width: f32) -> Flow {
    Flow {
        width,
        height: 0.2,
        spacing: width,
        nozzle_diameter: 0.4,
        bridge: false,
        mm3_per_mm: 1.0,
    }
}
