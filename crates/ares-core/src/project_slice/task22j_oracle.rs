use crate::geometry::{ExPolygon, Polygon};

use super::region_slices::{PostRegionPrintObject, RegionSurface};

pub(super) fn encode(objects: &[PostRegionPrintObject]) -> Vec<u8> {
    let mut output = b"ARES22J\0".to_vec();
    put_u64(&mut output, objects.len());
    for object in objects {
        let (plan, sidecars, regions) = object.as_parts();
        put_u64(&mut output, plan.source_object_index);
        put_u64(&mut output, plan.transform_index);
        put_u64(&mut output, plan.layers.len());
        put_u64(&mut output, sidecars.len());
        for sidecar in sidecars {
            let (occurrence_id, layers) = sidecar.as_parts();
            output.extend_from_slice(&u64::from(occurrence_id.get()).to_le_bytes());
            put_u64(&mut output, layers.len());
            for (layer_index, expolygons) in layers.iter().enumerate() {
                put_u64(&mut output, layer_index);
                encode_expolygons(&mut output, expolygons);
            }
        }
        put_u64(&mut output, plan.layers.len());
        for layer_index in 0..plan.layers.len() {
            put_u64(&mut output, layer_index);
            put_u64(&mut output, regions.len());
            for region in regions {
                let (region_id, _, layers) = region.as_parts();
                put_u64(&mut output, region_id);
                encode_surfaces(&mut output, layers[layer_index].surfaces());
            }
        }
    }
    output
}

fn encode_surfaces(output: &mut Vec<u8>, surfaces: &[RegionSurface]) {
    put_u64(output, surfaces.len());
    for surface in surfaces {
        let (kind, expolygon, ..) = surface.as_parts();
        output.push(kind as u8);
        encode_expolygon(output, expolygon);
    }
}

fn encode_expolygons(output: &mut Vec<u8>, expolygons: &[ExPolygon]) {
    put_u64(output, expolygons.len());
    for expolygon in expolygons {
        encode_expolygon(output, expolygon);
    }
}

fn encode_expolygon(output: &mut Vec<u8>, expolygon: &ExPolygon) {
    encode_polygon(output, expolygon.contour());
    put_u64(output, expolygon.holes().len());
    for hole in expolygon.holes() {
        encode_polygon(output, hole);
    }
}

fn encode_polygon(output: &mut Vec<u8>, polygon: &Polygon) {
    put_u64(output, polygon.points().len());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

fn put_u64(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap().to_le_bytes());
}
