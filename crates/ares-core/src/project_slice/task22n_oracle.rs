use crate::geometry::{ExPolygon, Polygon};

use super::{
    perimeters::types::{
        Flow, PerimeterDispatch, PerimeterInputRecord, PostPerimeterInputPrintObject,
    },
    region_slices::RegionSurface,
};

pub(super) fn encode(predecessor: &[u8], objects: &[PostPerimeterInputPrintObject]) -> Vec<u8> {
    let mut output = b"ARES22N\0".to_vec();
    put_usize(&mut output, predecessor.len());
    output.extend_from_slice(predecessor);
    put_usize(&mut output, objects.len());
    for object in objects {
        encode_object(&mut output, object);
    }
    output
}

fn encode_object(output: &mut Vec<u8>, object: &PostPerimeterInputPrintObject) {
    let (post_compensation, records) = object.as_parts();
    let (post_regions, layer_slices) = post_compensation.as_parts();
    let (plan, _, regions) = post_regions.as_parts();
    assert_eq!(records.len(), plan.layers.len());
    assert_eq!(layer_slices.len(), plan.layers.len());
    for region in regions {
        assert_eq!(region.as_parts().2.len(), plan.layers.len());
    }

    put_usize(output, plan.source_object_index);
    put_usize(output, plan.transform_index);
    put_usize(output, plan.layers.len());
    put_usize(output, records.len());
    for record in records {
        put_bool(output, record.is_some());
        if let Some(record) = record {
            encode_record(output, object, record);
        }
    }
}

fn encode_record(
    output: &mut Vec<u8>,
    object: &PostPerimeterInputPrintObject,
    record: &PerimeterInputRecord,
) {
    for value in [
        record.source_object_index,
        record.transform_index,
        record.planned_layer_index,
        record.layer_id,
        record.region_id,
    ] {
        put_usize(output, value);
    }
    put_usize(output, record.compatible_region_ids.len());
    for region_id in record.compatible_region_ids {
        put_usize(output, region_id);
    }
    put_usize(output, record.current.region_index);
    put_usize(output, record.current.layer_index);
    encode_optional_usize(output, record.lower_layer_index);
    encode_optional_usize(output, record.upper_layer_index);
    put_bool(output, record.upper_same_region.is_some());
    if let Some(index) = record.upper_same_region {
        put_usize(output, index.region_index);
        put_usize(output, index.layer_index);
    }

    encode_surfaces(output, object.current_surfaces(record));
    if let Some(slices) = object.lower_slices(record) {
        encode_expolygons(output, slices);
    }
    if let Some(slices) = object.upper_slices(record) {
        encode_expolygons(output, slices);
    }
    if let Some(surfaces) = object.upper_same_region_surfaces(record) {
        encode_surfaces(output, surfaces);
    }

    put_f64(output, record.layer_height);
    put_f64(output, record.slice_z);
    for flow in [
        record.perimeter_flow,
        record.ext_perimeter_flow,
        record.overhang_flow,
        record.solid_infill_flow,
    ] {
        encode_flow(output, flow);
    }
    put_bool(output, record.spiral_mode);
    put_f64(output, record.model_rotation_rad);
    output.push(match record.dispatch {
        PerimeterDispatch::Classic => 0,
        PerimeterDispatch::Arachne => 1,
    });
}

fn encode_flow(output: &mut Vec<u8>, flow: Flow) {
    for value in [flow.width, flow.height, flow.spacing, flow.nozzle_diameter] {
        output.extend_from_slice(&value.to_bits().to_le_bytes());
    }
    put_bool(output, flow.bridge);
    put_f64(output, flow.mm3_per_mm);
}

fn encode_surfaces(output: &mut Vec<u8>, surfaces: &[RegionSurface]) {
    put_usize(output, surfaces.len());
    for surface in surfaces {
        let (kind, expolygon, ..) = surface.as_parts();
        output.push(kind as u8);
        encode_expolygon(output, expolygon);
    }
}

fn encode_expolygons(output: &mut Vec<u8>, expolygons: &[ExPolygon]) {
    put_usize(output, expolygons.len());
    for expolygon in expolygons {
        encode_expolygon(output, expolygon);
    }
}

fn encode_expolygon(output: &mut Vec<u8>, expolygon: &ExPolygon) {
    encode_polygon(output, expolygon.contour());
    put_usize(output, expolygon.holes().len());
    for hole in expolygon.holes() {
        encode_polygon(output, hole);
    }
}

fn encode_polygon(output: &mut Vec<u8>, polygon: &Polygon) {
    put_usize(output, polygon.points().len());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

fn encode_optional_usize(output: &mut Vec<u8>, value: Option<usize>) {
    put_bool(output, value.is_some());
    if let Some(value) = value {
        put_usize(output, value);
    }
}

fn put_bool(output: &mut Vec<u8>, value: bool) {
    output.push(u8::from(value));
}

fn put_f64(output: &mut Vec<u8>, value: f64) {
    output.extend_from_slice(&value.to_bits().to_le_bytes());
}

fn put_usize(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap().to_le_bytes());
}
