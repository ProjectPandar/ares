use crate::{ProjectVolumeType, geometry::Polygon, mesh_slicer::SlicingMode};

use super::closing::{PostClosingLayer, PostClosingPrintObject};

#[cfg(test)]
pub(super) fn encode(objects: &[PostClosingPrintObject]) -> Vec<u8> {
    encode_with_magic(objects, b"ARES22G\0")
}

pub(super) fn encode_with_magic(objects: &[PostClosingPrintObject], magic: &[u8; 8]) -> Vec<u8> {
    let mut output = magic.to_vec();
    put_u64(&mut output, objects.len());
    for object in objects {
        put_u64(&mut output, object.plan().source_object_index);
        put_u64(&mut output, object.plan().transform_index);
        put_u64(&mut output, object.plan().layers.len());
        put_u64(&mut output, object.volumes().len());
        for volume in object.volumes() {
            put_u64(&mut output, volume.source_volume_index());
            output.extend_from_slice(&volume.ordinal().to_le_bytes());
            output.push(volume_type_code(volume.volume_type()));
            put_u64(&mut output, volume.layers().len());
            for (layer_index, layer) in volume.layers().iter().enumerate() {
                encode_layer(&mut output, layer_index, layer);
            }
        }
    }
    output
}

fn encode_layer(output: &mut Vec<u8>, index: usize, layer: &PostClosingLayer) {
    put_u64(output, index);
    output.push(mode_code(layer.mode()));
    put_u64(output, layer.expolygons().len());
    for expolygon in layer.expolygons() {
        encode_polygon(output, expolygon.contour());
        put_u64(output, expolygon.holes().len());
        for hole in expolygon.holes() {
            encode_polygon(output, hole);
        }
    }
}

fn encode_polygon(output: &mut Vec<u8>, polygon: &Polygon) {
    put_u64(output, polygon.points().len());
    for point in polygon.points() {
        output.extend_from_slice(&point.x().to_le_bytes());
        output.extend_from_slice(&point.y().to_le_bytes());
    }
}

const fn mode_code(mode: SlicingMode) -> u8 {
    match mode {
        SlicingMode::Regular => 0,
        SlicingMode::EvenOdd => 1,
        SlicingMode::Positive => 2,
        SlicingMode::PositiveLargestContour => 3,
    }
}

const fn volume_type_code(volume_type: ProjectVolumeType) -> u8 {
    match volume_type {
        ProjectVolumeType::ModelPart => 0,
        ProjectVolumeType::NegativeVolume => 1,
        ProjectVolumeType::ParameterModifier => 2,
        ProjectVolumeType::SupportEnforcer | ProjectVolumeType::SupportBlocker => {
            unreachable!()
        }
    }
}

fn put_u64(output: &mut Vec<u8>, value: usize) {
    output.extend_from_slice(&u64::try_from(value).unwrap().to_le_bytes());
}
