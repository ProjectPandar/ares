use crate::geometry::{ExPolygon, Polygon};

use super::{
    compensation::PostCompensationPrintObject,
    region_slices::{PostRegionPrintObject, RegionSurface},
};

pub(super) fn encode(objects: &[PostCompensationPrintObject]) -> Vec<u8> {
    let mut output = b"ARES22M\0".to_vec();
    put_u64(&mut output, objects.len());
    for object in objects {
        encode_object(&mut output, object);
    }
    output
}

fn encode_object(output: &mut Vec<u8>, object: &PostCompensationPrintObject) {
    let (post_regions, lslices) = object.as_parts();
    let (plan, sidecars, _) = post_regions.as_parts();
    assert_eq!(lslices.len(), plan.layers.len());
    put_u64(output, plan.source_object_index);
    put_u64(output, plan.transform_index);
    put_u64(output, plan.layers.len());
    put_u64(output, sidecars.len());
    for sidecar in sidecars {
        let (occurrence_id, layers) = sidecar.as_parts();
        output.extend_from_slice(&u64::from(occurrence_id.get()).to_le_bytes());
        put_u64(output, layers.len());
        for (layer_index, expolygons) in layers.iter().enumerate() {
            put_u64(output, layer_index);
            encode_expolygons(output, expolygons);
        }
    }
    encode_layers(output, post_regions, lslices);
}

fn encode_layers(output: &mut Vec<u8>, object: &PostRegionPrintObject, lslices: &[Vec<ExPolygon>]) {
    let (plan, _, regions) = object.as_parts();
    put_u64(output, plan.layers.len());
    for layer_index in 0..plan.layers.len() {
        put_u64(output, layer_index);
        put_u64(output, regions.len());
        for region in regions {
            let (region_id, _, layers) = region.as_parts();
            put_u64(output, region_id);
            encode_surfaces(output, layers[layer_index].surfaces());
        }
        encode_expolygons(output, &lslices[layer_index]);
    }
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
